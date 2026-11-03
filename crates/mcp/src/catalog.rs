//! 配方目录：每个 MCP 服务器一个 TOML 文件，声明「怎么装」与「怎么注册」
//!
//! 内置配方用 `include_str!` 编进二进制，另支持外部目录（默认 `~/.pengj-mcp/catalog`）
//! 按同名覆盖——改配方不必重编译，和模板层的思路一致。

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{McpError, Result};

/// 内置配方：名称 -> TOML 原文
const BUILTIN: &[(&str, &str)] = &[("renderdoc", include_str!("../catalog/renderdoc.toml"))];

/// 安装形态。决定 `add` 需要做什么，以及是否需要真的执行安装命令。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum InstallKind {
    /// 已在 PATH 上，或由用户自行安装：只写配置
    #[default]
    None,
    /// `uv tool install --from <git 源> <entry>`：PyPI 上没有包时走这条
    UvGit,
    /// `uvx <pkg>`：不落盘安装，客户端启动时临时拉取
    Uvx,
    /// `npx -y <pkg>`：同上，Node 生态
    Npm,
    /// 远程 MCP（Streamable HTTP / SSE）：只写地址
    Remote,
}

impl std::fmt::Display for InstallKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            InstallKind::None => "none",
            InstallKind::UvGit => "uv-git",
            InstallKind::Uvx => "uvx",
            InstallKind::Npm => "npm",
            InstallKind::Remote => "remote",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallSpec {
    #[serde(default)]
    pub kind: InstallKind,
    /// 安装源，如 `git+https://github.com/owner/repo` 或包名
    #[serde(default)]
    pub package: Option<String>,
    /// 安装产物/入口命令名，用于在 PATH 中定位可执行文件
    #[serde(default)]
    pub entry: Option<String>,
}

/// 启动参数：写到客户端配置里的「命令 + 参数」
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSpec {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// 远程服务器的端点（kind = remote 时使用）
    #[serde(default)]
    pub url: Option<String>,
}

/// 运行前提：缺了这些程序，MCP 装好也没数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Requires {
    #[serde(default)]
    pub apps: Vec<String>,
}

/// 只注册到部分客户端时用
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientScope {
    #[serde(default)]
    pub only: Vec<String>,
    #[serde(default)]
    pub skip: Vec<String>,
}

/// 附属产物（C 类安装）：主程序之外还需要拷进别的程序的目录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideSpec {
    /// 人读的名字，如「RenderDoc UI 扩展」
    pub what: String,
    /// 相对 `--source-root` 的源目录
    pub from: String,
    /// 目标目录（支持 `%VAR%` / `~`）
    pub to: String,
    #[serde(default)]
    pub hint: Option<String>,
}

/// 需要改写第三方程序配置才能生效的开关
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableSpec {
    pub what: String,
    /// 被改写的配置文件
    pub file: String,
    /// JSON 里的数组键名
    pub key: String,
    /// 追加进数组的值
    pub value: String,
}

/// 一条服务器配方
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRecipe {
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub install: InstallSpec,
    pub run: RunSpec,
    #[serde(default)]
    pub requires: Requires,
    #[serde(default)]
    pub clients: ClientScope,
    #[serde(default)]
    pub side: Vec<SideSpec>,
    #[serde(default)]
    pub enable: Vec<EnableSpec>,
    /// 安装后必须由人完成的事（GUI 点信任、重启程序等）
    #[serde(default)]
    pub manual_steps: Vec<String>,
}

impl ServerRecipe {
    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(McpError::InvalidCatalog("name 不能为空".into()));
        }
        if self.run.command.trim().is_empty() {
            return Err(McpError::InvalidCatalog(format!(
                "{}: run.command 不能为空",
                self.name
            )));
        }
        if self.install.kind == InstallKind::Remote && self.run.url.is_none() {
            return Err(McpError::InvalidCatalog(format!(
                "{}: kind = remote 时必须提供 run.url",
                self.name
            )));
        }
        if self.install.kind == InstallKind::UvGit
            && (self.install.package.is_none() || self.install.entry.is_none())
        {
            return Err(McpError::InvalidCatalog(format!(
                "{}: kind = uv-git 时必须提供 package 与 entry",
                self.name
            )));
        }
        Ok(())
    }

    /// 该配方是否应注册到指定客户端
    pub fn targets(&self, client_id: &str) -> bool {
        if self.clients.only.is_empty() {
            !self.clients.skip.iter().any(|c| c == client_id)
        } else {
            self.clients.only.iter().any(|c| c == client_id)
        }
    }
}

/// 配方集合
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    servers: Vec<ServerRecipe>,
}

impl Catalog {
    /// 加载内置配方，再叠加外部目录（同名覆盖）
    pub fn load(extra_dir: Option<&Path>) -> Result<Self> {
        let mut servers: Vec<ServerRecipe> = Vec::new();

        for (name, text) in BUILTIN {
            let recipe: ServerRecipe = toml::from_str(text)
                .map_err(|e| McpError::InvalidCatalog(format!("内置配方 {name}: {e}")))?;
            recipe.validate()?;
            servers.push(recipe);
        }

        if let Some(dir) = extra_dir {
            if dir.is_dir() {
                let mut files: Vec<_> = std::fs::read_dir(dir)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
                    .collect();
                files.sort();
                for file in files {
                    let text = std::fs::read_to_string(&file)?;
                    let recipe: ServerRecipe = toml::from_str(&text).map_err(|e| {
                        McpError::InvalidCatalog(format!("{}: {e}", file.display()))
                    })?;
                    recipe.validate()?;
                    servers.retain(|s| s.name != recipe.name);
                    servers.push(recipe);
                }
            }
        }

        servers.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self { servers })
    }

    pub fn servers(&self) -> &[ServerRecipe] {
        &self.servers
    }

    pub fn get(&self, name: &str) -> Option<&ServerRecipe> {
        self.servers.iter().find(|s| s.name == name)
    }

    pub fn require(&self, name: &str) -> Result<&ServerRecipe> {
        self.get(name).ok_or_else(|| {
            let all: Vec<&str> = self.servers.iter().map(|s| s.name.as_str()).collect();
            McpError::UnknownServer(format!("{name}（可用: {}）", all.join(", ")))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 内置配方可加载且字段完整() {
        let catalog = Catalog::load(None).expect("内置配方应能解析");
        let renderdoc = catalog.get("renderdoc").expect("应有 renderdoc 配方");
        assert_eq!(renderdoc.install.kind, InstallKind::UvGit);
        assert_eq!(renderdoc.run.command, "renderdoc-mcp");
        assert_eq!(
            renderdoc.install.package.as_deref(),
            Some("git+https://github.com/halby24/RenderDocMCP")
        );
        assert!(
            !renderdoc.side.is_empty(),
            "应声明 RenderDoc 扩展作为附属产物"
        );
        assert!(!renderdoc.manual_steps.is_empty(), "应声明人工步骤");
    }

    #[test]
    fn 外部配方按同名覆盖() {
        let dir = std::env::temp_dir().join(format!("pengj-mcp-cat-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("renderdoc.toml"),
            r#"
name = "renderdoc"
summary = "覆盖版"
[install]
kind = "none"
[run]
command = "custom-renderdoc"
"#,
        )
        .unwrap();

        let catalog = Catalog::load(Some(&dir)).unwrap();
        let r = catalog.get("renderdoc").unwrap();
        assert_eq!(r.summary, "覆盖版");
        assert_eq!(r.run.command, "custom-renderdoc");
        assert_eq!(
            catalog
                .servers()
                .iter()
                .filter(|s| s.name == "renderdoc")
                .count(),
            1,
            "同名配方不应重复"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 缺字段的配方报错() {
        let bad = r#"
name = "x"
summary = "y"
[install]
kind = "uv-git"
[run]
command = "x"
"#;
        let recipe: ServerRecipe = toml::from_str(bad).unwrap();
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn 客户端范围过滤生效() {
        let recipe: ServerRecipe = toml::from_str(
            r#"
name = "x"
summary = "y"
[run]
command = "x"
[clients]
skip = ["trae"]
"#,
        )
        .unwrap();
        assert!(recipe.targets("workbuddy"));
        assert!(!recipe.targets("trae"));
    }
}
