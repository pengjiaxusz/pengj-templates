//! 巡检：先做静态检查（配置解析、命令是否存在），再对每个唯一命令做一次真实握手
//!
//! 握手走标准 stdio JSON-RPC：`initialize` → `notifications/initialized` → `tools/list`。
//! 只认「进程能起且能应答」，不看文件是否存在——后者会漏掉「路径写死但已被删」的死链。

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};

use crate::client::{all_clients, RegisteredServer};
use crate::error::Result;
use crate::paths;
use crate::registry::resolve_command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    /// 命令可解析，配置正常
    Ok,
    /// 命令写成了路径，但文件已不存在
    DeadPath,
    /// 裸命令名，PATH 里找不到
    NotOnPath,
    /// 配置里被显式禁用
    Disabled,
    /// 既没有 command 也没有 url
    MissingCommand,
    /// 配置文件本身读不出来（JSON 损坏等）
    ConfigError,
}

impl CheckStatus {
    pub fn label(&self) -> &'static str {
        match self {
            CheckStatus::Ok => "正常",
            CheckStatus::DeadPath => "死链",
            CheckStatus::NotOnPath => "不在PATH",
            CheckStatus::Disabled => "已禁用",
            CheckStatus::MissingCommand => "无命令",
            CheckStatus::ConfigError => "读取失败",
        }
    }

    fn is_problem(&self) -> bool {
        !matches!(self, CheckStatus::Ok | CheckStatus::Disabled)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerCheck {
    pub client: String,
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    /// 实际用于握手的绝对路径（裸命令名已解析）
    pub resolved: Option<String>,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HandshakeCheck {
    pub target: String,
    pub ok: bool,
    pub server_name: Option<String>,
    pub server_version: Option<String>,
    pub tools: usize,
    pub elapsed_ms: u128,
    /// 握手成功但 tools/list 失败时的原因
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub checks: Vec<ServerCheck>,
    pub handshakes: Vec<HandshakeCheck>,
    pub fixed: Vec<String>,
}

impl DoctorReport {
    /// 问题数量：静态异常 + 握手失败
    pub fn problems(&self) -> usize {
        self.checks.iter().filter(|c| c.status.is_problem()).count()
            + self.handshakes.iter().filter(|h| !h.ok).count()
    }
}

#[derive(Debug, Clone)]
pub struct DoctorOptions {
    pub server: Option<String>,
    pub timeout: Duration,
    pub static_only: bool,
    pub fix: bool,
}

impl Default for DoctorOptions {
    fn default() -> Self {
        Self {
            server: None,
            timeout: Duration::from_secs(8),
            static_only: false,
            fix: false,
        }
    }
}

/// 巡检全部客户端；`fix` 会直接清掉死链条目
pub fn doctor(opts: &DoctorOptions) -> Result<DoctorReport> {
    let mut checks = Vec::new();
    let mut fixed = Vec::new();

    for client in all_clients() {
        let servers = match client.list() {
            Ok(s) => s,
            Err(e) => {
                checks.push(ServerCheck {
                    client: client.id().to_string(),
                    name: "-".into(),
                    command: None,
                    args: Vec::new(),
                    url: None,
                    resolved: None,
                    status: CheckStatus::ConfigError,
                    detail: format!("配置读取失败: {e}"),
                });
                continue;
            }
        };

        for server in servers {
            if let Some(filter) = &opts.server {
                if &server.name != filter {
                    continue;
                }
            }
            let mut check = classify(client.id(), &server);
            if opts.fix
                && matches!(check.status, CheckStatus::DeadPath | CheckStatus::NotOnPath)
                && client.remove(&server.name)?
            {
                fixed.push(format!("{}/{}", client.id(), server.name));
                check.detail = format!("已清理死链（原: {}）", check.detail);
                check.status = CheckStatus::Ok;
            }
            checks.push(check);
        }
    }

    let mut handshakes = Vec::new();
    if !opts.static_only {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for check in &checks {
            if check.status != CheckStatus::Ok || check.url.is_some() {
                continue;
            }
            let Some(target) = check.resolved.clone() else {
                continue;
            };
            let key = format!("{target}\u{1}{}", check.args.join("\u{1}"));
            if !seen.insert(key) {
                continue;
            }
            handshakes.push(run_handshake(&target, &check.args, opts.timeout));
        }
    }

    Ok(DoctorReport {
        checks,
        handshakes,
        fixed,
    })
}

fn classify(client: &str, server: &RegisteredServer) -> ServerCheck {
    let base = |status: CheckStatus, detail: String, resolved: Option<String>| ServerCheck {
        client: client.to_string(),
        name: server.name.clone(),
        command: server.command.clone(),
        args: server.args.clone(),
        url: server.url.clone(),
        resolved,
        status,
        detail,
    };

    if let Some(url) = &server.url {
        return base(CheckStatus::Ok, format!("远程 {url}（未做握手）"), None);
    }
    if !server.enabled {
        return base(CheckStatus::Disabled, "配置里被禁用".into(), None);
    }
    let Some(cmd) = server.command.clone() else {
        return base(CheckStatus::MissingCommand, "未声明 command".into(), None);
    };

    let looks_like_path = cmd.contains('\\') || cmd.contains('/');
    if looks_like_path {
        let resolved = paths::expand(&cmd);
        if resolved.is_file() {
            base(
                CheckStatus::Ok,
                resolved.display().to_string(),
                Some(resolved.display().to_string()),
            )
        } else {
            base(
                CheckStatus::DeadPath,
                format!("文件不存在: {}", resolved.display()),
                None,
            )
        }
    } else if let Some(found) = resolve_command(&cmd) {
        base(
            CheckStatus::Ok,
            found.display().to_string(),
            Some(found.display().to_string()),
        )
    } else {
        base(CheckStatus::NotOnPath, format!("PATH 中找不到 {cmd}"), None)
    }
}

struct HandshakeInfo {
    server_name: Option<String>,
    server_version: Option<String>,
    tools: usize,
    note: Option<String>,
}

fn run_handshake(command: &str, args: &[String], timeout: Duration) -> HandshakeCheck {
    let started = Instant::now();
    match handshake(command, args, timeout) {
        Ok(info) => HandshakeCheck {
            target: command.to_string(),
            ok: true,
            server_name: info.server_name,
            server_version: info.server_version,
            tools: info.tools,
            elapsed_ms: started.elapsed().as_millis(),
            note: info.note,
        },
        Err(error) => HandshakeCheck {
            target: command.to_string(),
            ok: false,
            server_name: None,
            server_version: None,
            tools: 0,
            elapsed_ms: started.elapsed().as_millis(),
            note: Some(error),
        },
    }
}

fn handshake(
    command: &str,
    args: &[String],
    timeout: Duration,
) -> std::result::Result<HandshakeInfo, String> {
    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn().map_err(|e| format!("无法启动进程: {e}"))?;

    let Some(stdin) = child.stdin.take() else {
        return Err("无法写入子进程 stdin".into());
    };
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        return Err("无法读取子进程 stdout".into());
    };

    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().map_while(|l| l.ok()) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    let err_tail: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    if let Some(stderr) = child.stderr.take() {
        let tail = Arc::clone(&err_tail);
        std::thread::spawn(move || {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines().map_while(|l| l.ok()) {
                if let Ok(mut buf) = tail.lock() {
                    if buf.len() >= 20 {
                        buf.remove(0);
                    }
                    buf.push(line);
                }
            }
        });
    }

    let result = drive(stdin, &rx, timeout);
    let _ = child.kill();
    let _ = child.wait();

    result.map_err(|e| {
        let tail = err_tail.lock().map(|t| t.join(" | ")).unwrap_or_default();
        if tail.is_empty() {
            e
        } else {
            format!("{e}（stderr: {tail}）")
        }
    })
}

/// 完成一次 initialize + tools/list 往返
fn drive(
    mut stdin: ChildStdin,
    rx: &Receiver<String>,
    timeout: Duration,
) -> std::result::Result<HandshakeInfo, String> {
    let init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "pengj-mcp-doctor", "version": env!("CARGO_PKG_VERSION") }
        }
    });
    send(&mut stdin, &init)?;

    let resp = wait_for(rx, 1, Instant::now() + timeout)?;
    let result = resp.get("result").cloned().unwrap_or(Value::Null);
    let server_name = result
        .get("serverInfo")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let server_version = result
        .get("serverInfo")
        .and_then(|v| v.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    send(
        &mut stdin,
        &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    )?;
    send(
        &mut stdin,
        &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }),
    )?;

    let (tools, note) = match wait_for(rx, 2, Instant::now() + timeout) {
        Ok(v) => {
            let count = v
                .get("result")
                .and_then(|r| r.get("tools"))
                .and_then(|t| t.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            (count, None)
        }
        Err(e) => (0, Some(format!("initialize 成功但 tools/list 失败: {e}"))),
    };

    Ok(HandshakeInfo {
        server_name,
        server_version,
        tools,
        note,
    })
}

fn send(stdin: &mut ChildStdin, payload: &Value) -> std::result::Result<(), String> {
    let line = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    writeln!(stdin, "{line}").map_err(|e| format!("写入子进程失败: {e}"))?;
    stdin
        .flush()
        .map_err(|e| format!("刷新子进程 stdin 失败: {e}"))
}

/// 等到指定 id 的应答；非 JSON 行（启动横幅、日志）直接跳过
fn wait_for(
    rx: &Receiver<String>,
    id: i64,
    deadline: Instant,
) -> std::result::Result<Value, String> {
    loop {
        let remain = deadline.saturating_duration_since(Instant::now());
        if remain.is_zero() {
            return Err(format!("等待 id={id} 的应答超时"));
        }
        match rx.recv_timeout(remain) {
            Ok(line) => {
                let Ok(value) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                if value.get("id").and_then(|v| v.as_i64()) != Some(id) {
                    continue;
                }
                if let Some(err) = value.get("error") {
                    return Err(format!("服务端返回错误: {err}"));
                }
                return Ok(value);
            }
            Err(RecvTimeoutError::Timeout) => return Err(format!("等待 id={id} 的应答超时")),
            Err(RecvTimeoutError::Disconnected) => {
                return Err("子进程提前退出（未产生应答）".into())
            }
        }
    }
}

/// 供 CLI 判断某条配置是否可握手（避免对远程地址发起 stdio）
pub fn is_stdio_checkable(check: &ServerCheck) -> bool {
    check.status == CheckStatus::Ok
        && check
            .resolved
            .as_deref()
            .is_some_and(|p| Path::new(p).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 状态标签与问题判定() {
        assert_eq!(CheckStatus::Ok.label(), "正常");
        assert_eq!(CheckStatus::DeadPath.label(), "死链");
        assert!(!CheckStatus::Disabled.is_problem(), "禁用不算故障");
        assert!(CheckStatus::DeadPath.is_problem());
    }

    #[test]
    fn 死链分类() {
        let server = RegisteredServer {
            name: "x".into(),
            command: Some("C:\\definitely\\missing\\mcp.exe".into()),
            args: vec![],
            url: None,
            enabled: true,
        };
        let check = classify("t", &server);
        assert_eq!(check.status, CheckStatus::DeadPath);
    }

    #[test]
    fn 裸命令解析后视为正常() {
        let server = RegisteredServer {
            name: "x".into(),
            command: Some(if cfg!(windows) { "cmd" } else { "sh" }.into()),
            args: vec![],
            url: None,
            enabled: true,
        };
        let check = classify("t", &server);
        assert_eq!(check.status, CheckStatus::Ok);
        assert!(is_stdio_checkable(&check));
    }

    #[test]
    fn 非json行被跳过() {
        let (tx, rx) = mpsc::channel();
        tx.send("FastMCP banner".to_string()).unwrap();
        tx.send(r#"{"jsonrpc":"2.0","id":7,"result":{"ok":true}}"#.to_string())
            .unwrap();
        let got = wait_for(&rx, 7, Instant::now() + Duration::from_secs(1)).unwrap();
        assert_eq!(got["result"]["ok"], json!(true));
    }

    #[test]
    fn 超时返回错误() {
        let (_tx, rx) = mpsc::channel::<String>();
        let err = wait_for(&rx, 1, Instant::now() + Duration::from_millis(30)).unwrap_err();
        assert!(err.contains("超时"));
    }
}
