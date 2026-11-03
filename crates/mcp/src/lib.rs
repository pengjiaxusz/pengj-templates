//! MCP 服务器自动安装与配置
//!
//! 与项目级的模板生成互补：这里的对象是「机器上的 MCP 客户端」，而不是仓库里的文件。
//!
//! - **配方目录**（[`catalog`]）：每个 MCP 服务器一份 TOML，声明安装形态与启动参数
//! - **客户端适配**（[`client`]）：同一份定义落到 WorkBuddy / opencode / Claude Code /
//!   Antigravity / Trae CN 各自的配置文件，写前备份、保留未知键、幂等
//! - **编排**（[`registry`]）：装主体 → 解析绝对路径 → 写配置 → 记 manifest
//! - **巡检**（[`doctor`]）：静态查死链，再对各唯一命令做一次真实 stdio 握手

pub mod catalog;
pub mod client;
pub mod doctor;
pub mod error;
pub mod manifest;
pub mod paths;
pub mod registry;

pub use catalog::{Catalog, InstallKind, ServerRecipe};
pub use client::{
    all_clients, client_ids, find_client, ClientAdapter, RegisteredServer, ServerSpec,
};
pub use doctor::{doctor, CheckStatus, DoctorOptions, DoctorReport, HandshakeCheck, ServerCheck};
pub use error::{McpError, Result};
pub use manifest::{InstalledServer, McpManifest};
pub use registry::{
    add, join_ids, remove, resolve_command, source_cache_dir, status_list, AddOptions, AddReport,
    ClientWrite, RemoveReport, ServerStatus, StepResult,
};
