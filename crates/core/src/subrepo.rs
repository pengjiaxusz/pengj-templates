//! 子仓库登记表（subrepo registry）
//!
//! 登记表是子仓库的**唯一事实源**：子仓库同步技能与体检命令读的是同一份文件，
//! 因此「技能建议升级哪些子仓库」与「自检检查哪些子仓库」永远对得上。
//!
//! 形状与提交门禁的配置同构：**受管说明块 + 用户条目**。受管说明块随模板刷新，
//! 用户条目逐字保留；模板只写块内的说明，块以下的 `[[subrepo]]` 表归项目所有。
//!
//! 依赖模式是**受约束的枚举**而非自由文本：三类子仓库（子模块接入 / 构建系统按版本
//! 拉取 / 本地同级目录）的升级与校验动作差别很大，模式必须是机器可读的。
//! 出现未知模式时解析**报错**而不是静默接受——静默跳过正是 Markdown 表格方案的失败方式。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

/// 登记表相对项目根的路径（模板产物目录内，不占用项目根命名空间）
pub const SUBREPO_REGISTRY_REL: &str = ".pengj-templates/subrepo.toml";

/// 子仓库的依赖模式：受约束的枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyMode {
    /// 以 git 子模块接入
    Submodule,
    /// 由宿主构建系统按版本拉取（FetchContent 式）
    Fetch,
    /// 本机上的同级目录
    Local,
}

impl DependencyMode {
    /// 模式的全集（用于错误信息里提示可用值）
    pub const ALL: [DependencyMode; 3] = [
        DependencyMode::Submodule,
        DependencyMode::Fetch,
        DependencyMode::Local,
    ];

    /// 模式的稳定字面量（写进登记表与命令输出的规范写法）
    pub fn as_str(self) -> &'static str {
        match self {
            DependencyMode::Submodule => "submodule",
            DependencyMode::Fetch => "fetch",
            DependencyMode::Local => "local",
        }
    }

    /// 解析模式字面量；未知模式是硬错误，不静默接受
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "submodule" => Ok(DependencyMode::Submodule),
            "fetch" => Ok(DependencyMode::Fetch),
            "local" => Ok(DependencyMode::Local),
            other => Err(CoreError::SubrepoRegistry(format!(
                "未知的依赖模式: {other}（可用值: {}）",
                DependencyMode::ALL
                    .iter()
                    .map(|m| m.as_str())
                    .collect::<Vec<_>>()
                    .join(" / ")
            ))),
        }
    }

    /// 面向人的模式说明（命令输出用）
    pub fn describe(self) -> &'static str {
        match self {
            DependencyMode::Submodule => "子模块接入",
            DependencyMode::Fetch => "构建系统按版本拉取",
            DependencyMode::Local => "本地同级目录",
        }
    }
}

/// 登记表里的一条子仓库登记
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubrepoEntry {
    /// 子仓库名称
    pub name: String,
    /// 相对项目根的路径
    pub path: String,
    /// 依赖模式
    pub mode: DependencyMode,
    /// 上游跟踪分支（`submodule` 模式必填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    /// 本地同级目录（`local` 模式必填）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_dir: Option<String>,
    /// 该子仓库是否自带工作区定义
    #[serde(default)]
    pub has_workspace: bool,
}

/// 一份登记表
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubrepoRegistry {
    /// 登记的子仓库条目
    #[serde(default, rename = "subrepo")]
    pub entries: Vec<SubrepoEntry>,
}

/// 登记表的反序列化形态：字段缺失或类型不符时给出可操作报错
#[derive(Debug, Deserialize)]
struct RawEntry {
    name: String,
    path: String,
    mode: String,
    #[serde(default)]
    upstream: Option<String>,
    #[serde(default)]
    local_dir: Option<String>,
    #[serde(default)]
    has_workspace: bool,
}

impl SubrepoRegistry {
    /// 从登记表文本解析。
    ///
    /// 解析失败即报错：未知依赖模式、缺少必填字段、空登记表都**不**降级为「跳过」。
    /// 体检命令依赖这份判定，静默跳过会让「检查通过」变成一句谎话。
    pub fn parse(text: &str) -> Result<Self> {
        // 登记表整体是一个 TOML 文档，顶层是 `subrepo` 数组表。
        // 受管说明块是 `#` 注释，TOML 解析器天然忽略，不影响结构化解析。
        #[derive(Deserialize)]
        struct RawRegistry {
            #[serde(default, rename = "subrepo")]
            subrepo: Vec<RawEntry>,
        }

        let raw: RawRegistry = toml::from_str(text)
            .map_err(|e| CoreError::SubrepoRegistry(format!("登记表不是合法的 TOML: {e}")))?;

        let mut entries = Vec::with_capacity(raw.subrepo.len());
        for (index, item) in raw.subrepo.into_iter().enumerate() {
            entries.push(Self::parse_entry(index, item)?);
        }
        Ok(Self { entries })
    }

    /// 解析单条登记并校验必填字段
    fn parse_entry(index: usize, item: RawEntry) -> Result<SubrepoEntry> {
        let position = index + 1;
        if item.name.trim().is_empty() {
            return Err(CoreError::SubrepoRegistry(format!(
                "第 {position} 条登记缺少 name（子仓库名称）"
            )));
        }
        if item.path.trim().is_empty() {
            return Err(CoreError::SubrepoRegistry(format!(
                "第 {position} 条登记（{}）缺少 path（相对项目根的路径）",
                item.name
            )));
        }
        let mode = DependencyMode::parse(item.mode.trim())?;
        match mode {
            DependencyMode::Submodule => {
                if item
                    .upstream
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .is_none()
                {
                    return Err(CoreError::SubrepoRegistry(format!(
                        "第 {position} 条登记（{}）的 mode 是 submodule，必须给出 upstream（上游跟踪分支）",
                        item.name
                    )));
                }
            }
            DependencyMode::Local => {
                if item
                    .local_dir
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .is_none()
                {
                    return Err(CoreError::SubrepoRegistry(format!(
                        "第 {position} 条登记（{}）的 mode 是 local，必须给出 local_dir（本地同级目录）",
                        item.name
                    )));
                }
            }
            DependencyMode::Fetch => {}
        }
        Ok(SubrepoEntry {
            name: item.name.trim().to_string(),
            path: item.path.trim().to_string(),
            mode,
            upstream: item
                .upstream
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            local_dir: item
                .local_dir
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            has_workspace: item.has_workspace,
        })
    }

    /// 从项目根读取登记表。
    ///
    /// 登记表不存在时返回 `Ok(None)`：项目可能压根没勾选子仓库层，
    /// 那不是错误；文件存在但解析失败则是错误，必须让调用方看见。
    pub fn load(project_dir: &Path) -> Result<Option<Self>> {
        let path = project_dir.join(SUBREPO_REGISTRY_REL);
        if !path.is_file() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path)?;
        Ok(Some(Self::parse(&text)?))
    }

    /// 登记表的绝对路径（项目根下）
    pub fn path_in(project_dir: &Path) -> PathBuf {
        project_dir.join(SUBREPO_REGISTRY_REL)
    }

    /// 按名称查找登记条目
    pub fn find(&self, name: &str) -> Option<&SubrepoEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_full_registry() {
        let text = r#"
# PENGJ_TEMPLATE_START
# 说明块
# PENGJ_TEMPLATE_END

[[subrepo]]
name = "shared-ui"
path = "submodules/shared-ui"
mode = "submodule"
upstream = "origin/main"
has_workspace = true

[[subrepo]]
name = "vendored-core"
path = "vendor/core"
mode = "fetch"

[[subrepo]]
name = "local-pe"
path = "external/pe"
mode = "local"
local_dir = "../pe"
"#;
        let registry = SubrepoRegistry::parse(text).expect("登记表应可解析");
        assert_eq!(registry.entries.len(), 3);

        let shared = registry.find("shared-ui").expect("应能按名查找");
        assert_eq!(shared.mode, DependencyMode::Submodule);
        assert_eq!(shared.upstream.as_deref(), Some("origin/main"));
        assert!(shared.has_workspace);

        let vendored = registry.find("vendored-core").expect("应能按名查找");
        assert_eq!(vendored.mode, DependencyMode::Fetch);
        assert!(vendored.upstream.is_none());

        let local = registry.find("local-pe").expect("应能按名查找");
        assert_eq!(local.mode, DependencyMode::Local);
        assert_eq!(local.local_dir.as_deref(), Some("../pe"));
    }

    #[test]
    fn empty_registry_parses_to_no_entries() {
        let registry = SubrepoRegistry::parse("# 只有说明\n").expect("空登记表应可解析");
        assert!(registry.entries.is_empty());
    }

    #[test]
    fn unknown_dependency_mode_is_a_hard_error() {
        let text = r#"
[[subrepo]]
name = "x"
path = "submodules/x"
mode = "vendor-drop"
"#;
        let error = SubrepoRegistry::parse(text).expect_err("未知模式必须报错");
        let message = error.to_string();
        assert!(
            message.contains("vendor-drop"),
            "报错应点明未知值: {message}"
        );
        assert!(
            message.contains("submodule") && message.contains("fetch") && message.contains("local"),
            "报错应列出全部可用模式: {message}"
        );
    }

    #[test]
    fn missing_required_fields_are_hard_errors() {
        let no_name = "[[subrepo]]\npath = \"p\"\nmode = \"fetch\"\n";
        assert!(SubrepoRegistry::parse(no_name).is_err(), "缺 name 必须报错");

        let no_path = "[[subrepo]]\nname = \"n\"\nmode = \"fetch\"\n";
        assert!(SubrepoRegistry::parse(no_path).is_err(), "缺 path 必须报错");

        let no_mode = "[[subrepo]]\nname = \"n\"\npath = \"p\"\n";
        assert!(SubrepoRegistry::parse(no_mode).is_err(), "缺 mode 必须报错");
    }

    #[test]
    fn mode_specific_requirements_are_enforced() {
        let submodule_without_upstream =
            "[[subrepo]]\nname = \"n\"\npath = \"p\"\nmode = \"submodule\"\n";
        assert!(
            SubrepoRegistry::parse(submodule_without_upstream).is_err(),
            "submodule 缺 upstream 必须报错"
        );

        let local_without_dir = "[[subrepo]]\nname = \"n\"\npath = \"p\"\nmode = \"local\"\n";
        assert!(
            SubrepoRegistry::parse(local_without_dir).is_err(),
            "local 缺 local_dir 必须报错"
        );
    }

    #[test]
    fn malformed_toml_is_a_hard_error() {
        let error =
            SubrepoRegistry::parse("[[subrepo]\nname = \"x\"\n").expect_err("非法 TOML 必须报错");
        assert!(
            error.to_string().contains("TOML"),
            "报错应说明是 TOML 问题: {error}"
        );
    }

    #[test]
    fn missing_registry_file_is_not_an_error() {
        let dir = std::env::temp_dir().join(format!(
            "pengj-subrepo-missing-{}-{}",
            std::process::id(),
            line!()
        ));
        std::fs::create_dir_all(&dir).expect("应能建临时目录");
        let loaded = SubrepoRegistry::load(&dir).expect("文件缺失不算错误");
        assert!(loaded.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dependency_mode_round_trips_through_its_literal() {
        for mode in DependencyMode::ALL {
            let parsed = DependencyMode::parse(mode.as_str()).expect("枚举自身应可解析");
            assert_eq!(parsed, mode);
        }
    }
}
