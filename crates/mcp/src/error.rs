//! 错误类型

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("未知的 MCP 服务器: {0}")]
    UnknownServer(String),

    #[error("未知的客户端: {0}（可用: {1}）")]
    UnknownClient(String, String),

    #[error("配方无效: {0}")]
    InvalidCatalog(String),

    #[error("客户端配置不是合法 JSON: {path} — {message}")]
    InvalidJson { path: String, message: String },

    #[error("命令未找到: {0}（不在 PATH 中，请先安装对应工具）")]
    CommandNotFound(String),

    #[error("客户端 {client} 不接受含空格的命令路径: {command}")]
    CommandHasSpace { client: String, command: String },

    #[error("无法定位配置目录: {0}")]
    NoConfigDir(String),

    #[error("渲染产物缺失: {0}")]
    MissingArtifact(String),

    #[error("外部命令执行失败: {command} — {message}")]
    CommandFailed { command: String, message: String },

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, McpError>;
