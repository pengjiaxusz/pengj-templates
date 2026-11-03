//! 已安装状态清单：`~/.pengj-mcp/manifest.json`
//!
//! 与项目级的 `.pengj-templates.json` 同思路：记录「装了什么、从哪来、写进了哪些客户端」，
//! `doctor` / `remove` 依赖它做增量判断。

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::paths;

pub const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpManifest {
    #[serde(default)]
    pub tool: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub servers: BTreeMap<String, InstalledServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledServer {
    pub name: String,
    /// 实际写入客户端配置的命令（通常是绝对路径）
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// 已注册的客户端 id
    #[serde(default)]
    pub clients: Vec<String>,
    /// 来源（配方里的 package，或 manual）
    #[serde(default)]
    pub source: String,
    /// 安装时间（Unix 秒）
    #[serde(default)]
    pub installed_at: u64,
}

impl McpManifest {
    pub fn path() -> Result<PathBuf> {
        Ok(paths::config_dir()?.join(MANIFEST_FILE))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self {
                tool: "pengj-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                servers: BTreeMap::new(),
            });
        }
        let text = std::fs::read_to_string(&path)?;
        if text.trim().is_empty() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut text = serde_json::to_string_pretty(self)?;
        text.push('\n');
        std::fs::write(path, text)?;
        Ok(())
    }

    /// 记录/更新一条服务器，并与已有客户端列表求并集
    pub fn record(&mut self, server: InstalledServer) {
        self.tool = "pengj-mcp".into();
        self.version = env!("CARGO_PKG_VERSION").into();
        match self.servers.get_mut(&server.name) {
            Some(existing) => {
                for c in &server.clients {
                    if !existing.clients.contains(c) {
                        existing.clients.push(c.clone());
                    }
                }
                existing.command = server.command;
                existing.args = server.args;
                existing.source = server.source;
                existing.installed_at = server.installed_at;
            }
            None => {
                self.servers.insert(server.name.clone(), server);
            }
        }
    }

    /// 从某条服务器里摘掉一个客户端；摘空则整条删除
    pub fn forget_client(&mut self, name: &str, client: &str) {
        if let Some(entry) = self.servers.get_mut(name) {
            entry.clients.retain(|c| c != client);
            if entry.clients.is_empty() {
                self.servers.remove(name);
            }
        }
    }

    pub fn forget(&mut self, name: &str) {
        self.servers.remove(name);
    }
}

/// 当前 Unix 时间戳（秒），用于安装时间戳
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
