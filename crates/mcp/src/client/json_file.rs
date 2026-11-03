//! 基于 JSON 文件的客户端适配器
//!
//! 覆盖四类同构配置（顶层 `mcpServers`）与 opencode 的特殊结构（`mcp.servers` + 命令数组）。

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use super::{ClientAdapter, RegisteredServer, ServerSpec};
use crate::error::{McpError, Result};
use crate::paths;

/// 条目在配置文件里的落点
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// 顶层 `mcpServers.<name>`
    McpServers,
    /// `mcp.servers.<name>`，且 command 为数组、带 type/enabled
    Opencode,
}

/// 配置文件的定位方式（延迟到调用时解析，便于构造期不依赖环境变量）
#[derive(Debug, Clone)]
enum Target {
    Home(&'static str),
    AppData(&'static str),
    /// 仅测试用：指向任意临时文件
    #[cfg(test)]
    Explicit(PathBuf),
}

pub struct JsonFileClient {
    id: &'static str,
    label: &'static str,
    target: Target,
    layout: Layout,
    post_steps: &'static [&'static str],
    /// Trae CN 的解析器不接受含空格的 command，提前拦截给出明确提示
    reject_space_in_command: bool,
}

impl JsonFileClient {
    /// WorkBuddy：`~/.workbuddy/mcp.json`，写完需在 GUI 里点一次「信任」
    pub fn workbuddy() -> Self {
        Self {
            id: "workbuddy",
            label: "WorkBuddy",
            target: Target::Home(".workbuddy/mcp.json"),
            layout: Layout::McpServers,
            post_steps: &["在 WorkBuddy 的 MCP 设置里对该服务器点一次「信任」，否则不会加载"],
            reject_space_in_command: false,
        }
    }

    /// opencode：`~/.config/opencode/opencode.json` 的 `mcp.servers`
    pub fn opencode() -> Self {
        Self {
            id: "opencode",
            label: "opencode",
            target: Target::Home(".config/opencode/opencode.json"),
            layout: Layout::Opencode,
            post_steps: &["opencode 需新开一个会话（或重启 TUI）才会加载新增的 MCP"],
            reject_space_in_command: false,
        }
    }

    /// Claude Code：`~/.claude.json` 顶层 `mcpServers`（文件里还有大量无关状态，必须原样保留）
    pub fn claude_code() -> Self {
        Self {
            id: "claude-code",
            label: "Claude Code",
            target: Target::Home(".claude.json"),
            layout: Layout::McpServers,
            post_steps: &["Claude Code 在下次启动会话时读取；也可用 `claude mcp list` 核对"],
            reject_space_in_command: false,
        }
    }

    /// Antigravity CLI（agy）：`~/.gemini/config/mcp_config.json`
    pub fn antigravity() -> Self {
        Self {
            id: "agy",
            label: "Antigravity CLI",
            target: Target::Home(".gemini/config/mcp_config.json"),
            layout: Layout::McpServers,
            post_steps: &["在 agy 里执行 /mcp reload（或重启 agy）让新配置生效"],
            reject_space_in_command: false,
        }
    }

    /// Trae CN：`%APPDATA%/Trae CN/User/mcp.json`
    pub fn trae() -> Self {
        Self {
            id: "trae",
            label: "Trae CN",
            target: Target::AppData("Trae CN/User/mcp.json"),
            layout: Layout::McpServers,
            post_steps: &["Trae CN 必须重启 IDE 才会重新加载 MCP 配置"],
            reject_space_in_command: true,
        }
    }

    #[cfg(test)]
    fn for_test(id: &'static str, path: PathBuf, layout: Layout) -> Self {
        Self {
            id,
            label: id,
            target: Target::Explicit(path),
            layout,
            post_steps: &[],
            reject_space_in_command: false,
        }
    }

    fn servers_path(&self) -> &'static [&'static str] {
        match self.layout {
            Layout::McpServers => &["mcpServers"],
            Layout::Opencode => &["mcp", "servers"],
        }
    }

    fn read_doc(&self) -> Result<Value> {
        let path = self.config_path()?;
        if !path.exists() {
            return Ok(Value::Object(Map::new()));
        }
        let text = std::fs::read_to_string(&path)?;
        if text.trim().is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_str(&text).map_err(|e| McpError::InvalidJson {
            path: path.display().to_string(),
            message: e.to_string(),
        })
    }

    fn write_doc(&self, doc: &Value) -> Result<()> {
        let path = self.config_path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        if path.exists() {
            let bak = backup_path(&path);
            if !bak.exists() {
                std::fs::copy(&path, &bak)?;
            }
        }
        let mut text = serde_json::to_string_pretty(doc)?;
        text.push('\n');
        std::fs::write(&path, text)?;
        Ok(())
    }

    fn servers_ref<'a>(&self, doc: &'a Value) -> Option<&'a Map<String, Value>> {
        let mut cur = doc;
        for key in self.servers_path() {
            cur = cur.get(key)?;
        }
        cur.as_object()
    }

    fn servers_mut<'a>(&self, doc: &'a mut Value) -> Result<&'a mut Map<String, Value>> {
        let not_object = || McpError::InvalidCatalog("客户端配置的顶层不是 JSON 对象".into());
        match self.layout {
            Layout::McpServers => {
                let root = doc.as_object_mut().ok_or_else(not_object)?;
                let node = root
                    .entry("mcpServers".to_string())
                    .or_insert_with(|| Value::Object(Map::new()));
                node.as_object_mut()
                    .ok_or_else(|| McpError::InvalidCatalog("mcpServers 不是 JSON 对象".into()))
            }
            Layout::Opencode => {
                let root = doc.as_object_mut().ok_or_else(not_object)?;
                let mcp = root
                    .entry("mcp".to_string())
                    .or_insert_with(|| Value::Object(Map::new()));
                let mcp = mcp
                    .as_object_mut()
                    .ok_or_else(|| McpError::InvalidCatalog("mcp 不是 JSON 对象".into()))?;
                let servers = mcp
                    .entry("servers".to_string())
                    .or_insert_with(|| Value::Object(Map::new()));
                servers
                    .as_object_mut()
                    .ok_or_else(|| McpError::InvalidCatalog("mcp.servers 不是 JSON 对象".into()))
            }
        }
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "config".to_string());
    path.with_file_name(format!("{name}.pengj-mcp.bak"))
}

/// 按平台分隔符拼接相对路径，避免出现 `C:\a/.b/c` 这种混合写法
fn join_rel(base: PathBuf, rel: &str) -> PathBuf {
    rel.split('/')
        .filter(|s| !s.is_empty())
        .fold(base, |acc, seg| acc.join(seg))
}

impl ClientAdapter for JsonFileClient {
    fn id(&self) -> &'static str {
        self.id
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn config_path(&self) -> Result<PathBuf> {
        match &self.target {
            Target::Home(rel) => Ok(join_rel(paths::home_dir()?, rel)),
            Target::AppData(rel) => Ok(join_rel(paths::appdata_dir()?, rel)),
            #[cfg(test)]
            Target::Explicit(p) => Ok(p.clone()),
        }
    }

    fn list(&self) -> Result<Vec<RegisteredServer>> {
        let doc = self.read_doc()?;
        let Some(map) = self.servers_ref(&doc) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::with_capacity(map.len());
        for (name, entry) in map {
            let (command, args) = match self.layout {
                Layout::McpServers => (
                    entry
                        .get("command")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    string_array(entry.get("args")),
                ),
                Layout::Opencode => {
                    let mut cmd = string_array(entry.get("command"));
                    if cmd.is_empty() {
                        (None, Vec::new())
                    } else {
                        let command = cmd.remove(0);
                        (Some(command), cmd)
                    }
                }
            };
            let url = entry
                .get("serverUrl")
                .or_else(|| entry.get("url"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let enabled = match self.layout {
                Layout::McpServers => !entry
                    .get("disabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                Layout::Opencode => entry
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            };
            out.push(RegisteredServer {
                name: name.clone(),
                command,
                args,
                url,
                enabled,
            });
        }
        Ok(out)
    }

    fn render(&self, spec: &ServerSpec) -> Result<Value> {
        if let (Some(cmd), true) = (&spec.command, self.reject_space_in_command) {
            if cmd.contains(' ') {
                return Err(McpError::CommandHasSpace {
                    client: self.label.to_string(),
                    command: cmd.clone(),
                });
            }
        }
        let entry = match self.layout {
            Layout::McpServers => {
                let mut obj = Map::new();
                if let Some(cmd) = &spec.command {
                    obj.insert("command".into(), json!(cmd));
                    obj.insert("args".into(), json!(spec.args));
                }
                if let Some(url) = &spec.url {
                    obj.insert("serverUrl".into(), json!(url));
                }
                if !spec.env.is_empty() {
                    obj.insert("env".into(), json!(spec.env));
                }
                Value::Object(obj)
            }
            Layout::Opencode => {
                let mut obj = Map::new();
                if let Some(url) = &spec.url {
                    obj.insert("type".into(), json!("remote"));
                    obj.insert("url".into(), json!(url));
                } else {
                    obj.insert("type".into(), json!("local"));
                    let mut cmd = Vec::with_capacity(spec.args.len() + 1);
                    if let Some(c) = &spec.command {
                        cmd.push(Value::String(c.clone()));
                    }
                    for a in &spec.args {
                        cmd.push(Value::String(a.clone()));
                    }
                    obj.insert("command".into(), Value::Array(cmd));
                }
                obj.insert("enabled".into(), json!(true));
                if !spec.env.is_empty() {
                    obj.insert("environment".into(), json!(spec.env));
                }
                Value::Object(obj)
            }
        };
        Ok(entry)
    }

    fn upsert(&self, spec: &ServerSpec) -> Result<bool> {
        let rendered = self.render(spec)?;
        let mut doc = self.read_doc()?;
        let unchanged = self
            .servers_ref(&doc)
            .and_then(|m| m.get(&spec.name))
            .is_some_and(|v| v == &rendered);
        if unchanged {
            return Ok(false);
        }
        {
            let servers = self.servers_mut(&mut doc)?;
            servers.insert(spec.name.clone(), rendered);
        }
        self.write_doc(&doc)?;
        Ok(true)
    }

    fn remove(&self, name: &str) -> Result<bool> {
        let mut doc = self.read_doc()?;
        let existed = self.servers_ref(&doc).is_some_and(|m| m.contains_key(name));
        if !existed {
            return Ok(false);
        }
        {
            let servers = self.servers_mut(&mut doc)?;
            servers.remove(name);
        }
        self.write_doc(&doc)?;
        Ok(true)
    }

    fn post_steps(&self) -> Vec<String> {
        self.post_steps.iter().map(|s| s.to_string()).collect()
    }
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn temp_path(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pengj-mcp-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("mcp.json")
    }

    fn spec(name: &str, cmd: &str) -> ServerSpec {
        ServerSpec {
            name: name.to_string(),
            command: Some(cmd.to_string()),
            args: vec![],
            env: BTreeMap::new(),
            url: None,
        }
    }

    #[test]
    fn 顶层结构写入并保留未知键() {
        let path = temp_path("root");
        std::fs::write(
            &path,
            r#"{
  "mcpServers": { "other": { "command": "x" } },
  "unrelated": { "keep": true }
}"#,
        )
        .unwrap();
        let client = JsonFileClient::for_test("t", path.clone(), Layout::McpServers);

        assert!(client.upsert(&spec("renderdoc", "C:\\bin\\r.exe")).unwrap());
        assert!(
            !client.upsert(&spec("renderdoc", "C:\\bin\\r.exe")).unwrap(),
            "同内容应幂等"
        );

        let text = std::fs::read_to_string(&path).unwrap();
        let doc: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(doc["unrelated"]["keep"], json!(true), "无关键必须保留");
        assert_eq!(doc["mcpServers"]["other"]["command"], json!("x"));
        assert_eq!(
            doc["mcpServers"]["renderdoc"]["command"],
            json!("C:\\bin\\r.exe")
        );
        assert_eq!(doc["mcpServers"]["renderdoc"]["args"], json!([]));
        assert!(backup_path(&path).exists(), "改写前应留备份");

        let listed = client.list().unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().any(|s| s.name == "renderdoc" && s.enabled));
    }

    #[test]
    fn opencode_结构命令为数组() {
        let path = temp_path("opencode");
        std::fs::write(
            &path,
            r#"{
  "mcp": { "servers": { "old": { "type": "local", "command": ["old-exe"], "enabled": false } } },
  "provider": { "x": 1 }
}"#,
        )
        .unwrap();
        let client = JsonFileClient::for_test("opencode", path.clone(), Layout::Opencode);
        let mut s = spec("renderdoc", "D:\\shim\\renderdoc-mcp.exe");
        s.args = vec!["--verbose".into()];

        assert!(client.upsert(&s).unwrap());
        let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(doc["provider"]["x"], json!(1), "其它顶层段必须保留");
        assert_eq!(doc["mcp"]["servers"]["renderdoc"]["type"], json!("local"));
        assert_eq!(
            doc["mcp"]["servers"]["renderdoc"]["command"],
            json!(["D:\\shim\\renderdoc-mcp.exe", "--verbose"])
        );
        assert_eq!(doc["mcp"]["servers"]["old"]["enabled"], json!(false));

        let listed = client.list().unwrap();
        let rd = listed.iter().find(|s| s.name == "renderdoc").unwrap();
        assert_eq!(rd.command.as_deref(), Some("D:\\shim\\renderdoc-mcp.exe"));
        assert_eq!(rd.args, vec!["--verbose".to_string()]);
    }

    #[test]
    fn 删除条目且其余不动() {
        let path = temp_path("remove");
        let client = JsonFileClient::for_test("t", path.clone(), Layout::McpServers);
        client.upsert(&spec("a", "x")).unwrap();
        client.upsert(&spec("b", "y")).unwrap();
        assert!(client.remove("a").unwrap());
        assert!(!client.remove("a").unwrap(), "重复删除应返回无改动");
        let listed = client.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "b");
    }

    #[test]
    fn 含空格的命令被拒绝() {
        let path = temp_path("space");
        let mut client = JsonFileClient::for_test("trae", path, Layout::McpServers);
        client.reject_space_in_command = true;
        let err = client
            .upsert(&spec("x", "C:\\Program Files\\r\\r.exe"))
            .unwrap_err();
        assert!(
            matches!(err, McpError::CommandHasSpace { .. }),
            "应报空格错误"
        );
    }

    #[test]
    fn 非法json给出文件路径() {
        let path = temp_path("bad");
        std::fs::write(&path, "{ not json").unwrap();
        let client = JsonFileClient::for_test("t", path.clone(), Layout::McpServers);
        let err = client.list().unwrap_err();
        assert!(err.to_string().contains("mcp.json"), "错误信息应含文件路径");
    }

    #[test]
    fn 文件不存在时按空配置处理() {
        let dir = std::env::temp_dir().join(format!("pengj-mcp-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let client = JsonFileClient::for_test("t", dir.join("nested/mcp.json"), Layout::McpServers);
        assert!(client.list().unwrap().is_empty());
        assert!(
            client.upsert(&spec("a", "x")).unwrap(),
            "应自动建目录并写入"
        );
    }
}
