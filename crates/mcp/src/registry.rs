//! 安装与注册编排：解析配方 → 装主体 → 解析绝对路径 → 写各客户端配置

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use serde_json::Value;

use crate::catalog::{Catalog, EnableSpec, InstallKind, ServerRecipe, SideSpec};
use crate::client::{all_clients, find_client, ServerSpec};
use crate::error::{McpError, Result};
use crate::manifest::{now_unix, InstalledServer, McpManifest};
use crate::paths;

/// `add` 的选项
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    /// 目标客户端 id；空表示全部
    pub clients: Vec<String>,
    /// 只写配置，跳过主体安装
    pub skip_install: bool,
    /// 只打印计划，不落盘
    pub dry_run: bool,
    /// 附属产物的源目录（仓库副本根目录）
    pub source_root: Option<PathBuf>,
    /// 是否执行附属产物拷贝
    pub with_side: bool,
    /// 是否改写第三方程序配置以启用（如 RenderDoc 的 AlwaysLoad_Extensions）
    pub enable_app: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientWrite {
    pub client: String,
    pub label: String,
    pub path: String,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepResult {
    pub what: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddReport {
    pub name: String,
    pub summary: String,
    pub command: String,
    pub install_state: String,
    pub writes: Vec<ClientWrite>,
    pub skipped_clients: Vec<String>,
    pub steps: Vec<StepResult>,
    pub manual_steps: Vec<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveReport {
    pub name: String,
    pub writes: Vec<ClientWrite>,
    pub purged: bool,
}

/// 一条配方在本机的状态（供 `mcp list`）
#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub name: String,
    pub summary: String,
    pub install_kind: String,
    pub installed: bool,
    pub clients: Vec<String>,
}

/// 安装并注册一条配方
pub fn add(catalog: &Catalog, name: &str, opts: &AddOptions) -> Result<AddReport> {
    let recipe = catalog.require(name)?.clone();

    let (install_state, installed_now) = ensure_body(&recipe, opts)?;
    let command = resolve_run_command(&recipe, opts, installed_now)?;

    let spec = ServerSpec {
        name: recipe.name.clone(),
        command: Some(command.clone()),
        args: recipe.run.args.clone(),
        env: recipe.run.env.clone(),
        url: recipe.run.url.clone(),
    };

    let wanted = if opts.clients.is_empty() {
        all_clients().iter().map(|c| c.id().to_string()).collect()
    } else {
        opts.clients.clone()
    };

    let mut writes = Vec::new();
    let mut skipped_clients = Vec::new();
    for id in &wanted {
        if !recipe.targets(id) {
            skipped_clients.push(id.clone());
            continue;
        }
        let client = find_client(id)?;
        let path = client.config_path()?;
        let changed = if opts.dry_run {
            false
        } else {
            client.upsert(&spec)?
        };
        writes.push(ClientWrite {
            client: client.id().to_string(),
            label: client.label().to_string(),
            path: path.display().to_string(),
            changed,
        });
    }

    let mut steps: Vec<StepResult> = Vec::new();
    if opts.with_side {
        for side in &recipe.side {
            steps.push(apply_side(side, opts)?);
        }
    }
    if opts.enable_app {
        for spec in &recipe.enable {
            steps.push(apply_enable(&recipe, spec, opts.dry_run)?);
        }
    }

    if !opts.dry_run {
        let mut manifest = McpManifest::load()?;
        manifest.record(InstalledServer {
            name: recipe.name.clone(),
            command,
            args: recipe.run.args.clone(),
            clients: writes.iter().map(|w| w.client.clone()).collect(),
            source: recipe
                .install
                .package
                .clone()
                .unwrap_or_else(|| "manual".into()),
            installed_at: now_unix(),
        });
        manifest.save()?;
    }

    Ok(AddReport {
        name: recipe.name,
        summary: recipe.summary,
        command: spec.command.unwrap_or_default(),
        install_state,
        writes,
        skipped_clients,
        steps,
        manual_steps: recipe.manual_steps,
        dry_run: opts.dry_run,
    })
}

/// 从所有客户端移除，可选卸载主体
pub fn remove(catalog: &Catalog, name: &str, purge: bool) -> Result<RemoveReport> {
    let mut writes = Vec::new();
    for client in all_clients() {
        let path = client.config_path()?;
        let changed = client.remove(name)?;
        writes.push(ClientWrite {
            client: client.id().to_string(),
            label: client.label().to_string(),
            path: path.display().to_string(),
            changed,
        });
    }

    let mut purged = false;
    if purge {
        if let Some(recipe) = catalog.get(name) {
            if recipe.install.kind == InstallKind::UvGit {
                if let Some(entry) = recipe.install.entry.as_deref() {
                    let status = Command::new("uv")
                        .args(["tool", "uninstall", entry])
                        .status()?;
                    purged = status.success();
                }
            }
        }
    }

    let mut manifest = McpManifest::load()?;
    manifest.forget(name);
    manifest.save()?;

    Ok(RemoveReport {
        name: name.to_string(),
        writes,
        purged,
    })
}

/// 列出配方 + 本机安装状态
pub fn status_list(catalog: &Catalog) -> Result<Vec<ServerStatus>> {
    let clients = all_clients();
    let mut out = Vec::new();
    for recipe in catalog.servers() {
        let registered: Vec<String> = clients
            .iter()
            .filter(|c| {
                c.list()
                    .map(|servers| servers.iter().any(|s| s.name == recipe.name))
                    .unwrap_or(false)
            })
            .map(|c| c.id().to_string())
            .collect();

        let installed = match recipe.install.kind {
            InstallKind::UvGit => recipe
                .install
                .entry
                .as_deref()
                .map(|entry| uv_tool_installed(entry) || resolve_command(entry).is_some())
                .unwrap_or(false),
            InstallKind::Remote => true,
            _ => resolve_command(&recipe.run.command).is_some(),
        };

        out.push(ServerStatus {
            name: recipe.name.clone(),
            summary: recipe.summary.clone(),
            install_kind: recipe.install.kind.to_string(),
            installed,
            clients: registered,
        });
    }
    Ok(out)
}

/// 确保主体可用，返回（状态描述, 本次是否新装）
fn ensure_body(recipe: &ServerRecipe, opts: &AddOptions) -> Result<(String, bool)> {
    match recipe.install.kind {
        InstallKind::UvGit => {
            let entry = recipe
                .install
                .entry
                .as_deref()
                .unwrap_or(&recipe.run.command);
            if uv_tool_installed(entry) || resolve_command(entry).is_some() {
                return Ok(("已存在（跳过安装）".into(), false));
            }
            if opts.skip_install {
                return Ok(("跳过安装（--skip-install）".into(), false));
            }
            if opts.dry_run {
                return Ok(("预演：将执行 uv tool install".into(), false));
            }
            let package = recipe.install.package.as_deref().ok_or_else(|| {
                McpError::InvalidCatalog(format!("{}: 缺少 install.package", recipe.name))
            })?;
            install_uv_git(package, entry)?;
            Ok((format!("新装（uv tool install {entry}）"), true))
        }
        InstallKind::Uvx | InstallKind::Npm | InstallKind::None => {
            Ok(("按需拉取/已就绪（不落盘安装）".into(), false))
        }
        InstallKind::Remote => Ok(("远程服务（无需安装）".into(), false)),
    }
}

/// 解析要写进客户端配置的命令：优先绝对路径（GUI 客户端继承的 PATH 往往不全）
fn resolve_run_command(
    recipe: &ServerRecipe,
    opts: &AddOptions,
    installed_now: bool,
) -> Result<String> {
    if recipe.install.kind == InstallKind::Remote {
        return Ok(recipe.run.command.clone());
    }
    let raw = recipe.run.command.as_str();
    if let Some(found) = resolve_command(raw) {
        return Ok(found.display().to_string());
    }
    if installed_now || opts.dry_run || opts.skip_install {
        return Ok(raw.to_string());
    }
    Err(McpError::CommandNotFound(raw.to_string()))
}

/// 在 PATH（以及 uv 的 shim 目录）里查找可执行文件
pub fn resolve_command(name: &str) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }
    if name.contains('/') || name.contains('\\') {
        let direct = paths::expand(name);
        return direct.is_file().then_some(direct);
    }

    // Windows 上优先找真正的可执行文件：npm 的 shim 目录里同时存在
    // 无扩展名的 shell 脚本与 .cmd，前者无法被 CreateProcess 直接启动
    let exts: &[&str] = if cfg!(windows) {
        &[".exe", ".cmd", ".bat", ""]
    } else {
        &[""]
    };

    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|v| std::env::split_paths(&v).collect())
        .unwrap_or_default();
    if let Ok(home) = paths::home_dir() {
        dirs.push(home.join(".local/bin"));
    }
    for dir in dirs {
        if let Some(found) = probe_dir(&dir, name, exts) {
            return Some(found);
        }
    }

    if let Some(bin) = uv_tool_bin_dir() {
        if let Some(found) = probe_dir(&bin, name, exts) {
            return Some(found);
        }
    }
    None
}

fn probe_dir(dir: &Path, name: &str, exts: &[&str]) -> Option<PathBuf> {
    exts.iter()
        .map(|ext| dir.join(format!("{name}{ext}")))
        .find(|cand| cand.is_file())
}

/// `uv tool dir --bin`，uv 的 shim 目录不一定在 PATH 上
fn uv_tool_bin_dir() -> Option<PathBuf> {
    let out = Command::new("uv")
        .args(["tool", "dir", "--bin"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(PathBuf::from(text))
    }
}

/// `uv tool list` 里是否已有该工具
pub fn uv_tool_installed(entry: &str) -> bool {
    let Ok(out) = Command::new("uv").args(["tool", "list"]).output() else {
        return false;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines().any(|line| {
        line.trim_start_matches(['-', ' '])
            .split_whitespace()
            .next()
            .is_some_and(|token| token == entry)
    })
}

fn install_uv_git(package: &str, entry: &str) -> Result<()> {
    let status = Command::new("uv")
        .args(["tool", "install", "--force", "--from", package, entry])
        .status()?;
    if !status.success() {
        return Err(McpError::CommandFailed {
            command: format!("uv tool install --from {package} {entry}"),
            message: format!("退出码 {:?}", status.code()),
        });
    }
    Ok(())
}

/// 拷贝附属产物（C 类：主程序之外还要落进别的程序目录）
fn apply_side(side: &SideSpec, opts: &AddOptions) -> Result<StepResult> {
    let hint = side.hint.clone().unwrap_or_default();
    let Some(root) = &opts.source_root else {
        return Ok(StepResult {
            what: side.what.clone(),
            ok: false,
            detail: format!("未提供 --source-root，跳过拷贝。{hint}"),
        });
    };
    let from = root.join(&side.from);
    let to = paths::expand(&side.to);
    if !from.is_dir() {
        return Ok(StepResult {
            what: side.what.clone(),
            ok: false,
            detail: format!("源目录不存在: {}。{hint}", from.display()),
        });
    }
    if opts.dry_run {
        return Ok(StepResult {
            what: side.what.clone(),
            ok: true,
            detail: format!("预演：{} → {}", from.display(), to.display()),
        });
    }
    if to.exists() {
        std::fs::remove_dir_all(&to)?;
    }
    copy_dir_all(&from, &to)?;
    Ok(StepResult {
        what: side.what.clone(),
        ok: true,
        detail: format!("已拷贝到 {}", to.display()),
    })
}

fn copy_dir_all(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "__pycache__" || name_str.ends_with(".pyc") {
            continue;
        }
        let dest = to.join(&name);
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else if ty.is_file() {
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

/// 改写第三方程序配置以启用（如 RenderDoc 的 `AlwaysLoad_Extensions`）
fn apply_enable(recipe: &ServerRecipe, spec: &EnableSpec, dry_run: bool) -> Result<StepResult> {
    let file = paths::expand(&spec.file);
    if !file.exists() {
        return Ok(StepResult {
            what: spec.what.clone(),
            ok: false,
            detail: format!("配置文件不存在，需先启动一次目标程序: {}", file.display()),
        });
    }
    for app in &recipe.requires.apps {
        let image = if app.ends_with(".exe") {
            app.clone()
        } else {
            format!("{app}.exe")
        };
        if process_running(&image) {
            return Ok(StepResult {
                what: spec.what.clone(),
                ok: false,
                detail: format!("{image} 正在运行，退出时会覆盖该配置；请先关闭再执行"),
            });
        }
    }

    let text = std::fs::read_to_string(&file)?;
    let mut doc: Value = serde_json::from_str(&text).map_err(|e| McpError::InvalidJson {
        path: file.display().to_string(),
        message: e.to_string(),
    })?;

    let already = doc
        .get(&spec.key)
        .and_then(|v| v.as_array())
        .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(&spec.value)));

    if dry_run {
        return Ok(StepResult {
            what: spec.what.clone(),
            ok: true,
            detail: if already {
                "预演：已在列表中".into()
            } else {
                format!(
                    "预演：将把 {} 追加到 {} 的 {}",
                    spec.value,
                    file.display(),
                    spec.key
                )
            },
        });
    }
    if already {
        return Ok(StepResult {
            what: spec.what.clone(),
            ok: true,
            detail: "已在列表中，无需改动".into(),
        });
    }

    let root = doc
        .as_object_mut()
        .ok_or_else(|| McpError::InvalidCatalog("目标配置的顶层不是 JSON 对象".into()))?;
    match root.get_mut(&spec.key) {
        Some(Value::Array(arr)) => arr.push(Value::String(spec.value.clone())),
        Some(_) => {
            return Ok(StepResult {
                what: spec.what.clone(),
                ok: false,
                detail: format!("{} 不是数组，未改动", spec.key),
            });
        }
        None => {
            root.insert(
                spec.key.clone(),
                Value::Array(vec![Value::String(spec.value.clone())]),
            );
        }
    }
    let mut out = serde_json::to_string_pretty(&doc)?;
    out.push('\n');
    std::fs::write(&file, out)?;
    Ok(StepResult {
        what: spec.what.clone(),
        ok: true,
        detail: format!("已写入 {}", file.display()),
    })
}

#[cfg(windows)]
fn process_running(image: &str) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {image}"), "/NH"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .contains(&image.to_lowercase())
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn process_running(image: &str) -> bool {
    Command::new("pgrep")
        .args(["-f", image])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 供报告输出：把客户端 id 列表拼成一行
pub fn join_ids(ids: &[String]) -> String {
    if ids.is_empty() {
        "-".to_string()
    } else {
        ids.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 解析已存在的命令() {
        let found = resolve_command(if cfg!(windows) { "cmd" } else { "sh" });
        assert!(found.is_some(), "系统自带 shell 应能被解析");
    }

    #[test]
    fn 不存在的命令返回none() {
        assert!(resolve_command("pengj-definitely-not-a-command").is_none());
    }

    #[test]
    fn 路径形式直接判存在() {
        let exe = std::env::current_exe().unwrap();
        assert!(resolve_command(&exe.display().to_string()).is_some());
    }
}
