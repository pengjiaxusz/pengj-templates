//! 客户端适配：把同一份 `ServerSpec` 落到各家不同的配置文件结构里
//!
//! 五家的差异集中在三处：文件路径、条目所在层级、命令的写法（字符串还是数组）。
//! 其余（写前备份、保留未知键、幂等）都在这里统一处理。

use std::path::PathBuf;

use crate::error::Result;

pub mod json_file;

pub use json_file::{JsonFileClient, Layout};

/// 待写入客户端的服务器定义（已归一化，与具体客户端无关）
#[derive(Debug, Clone)]
pub struct ServerSpec {
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: std::collections::BTreeMap<String, String>,
    pub url: Option<String>,
}

/// 客户端配置里已存在的一条服务器
#[derive(Debug, Clone)]
pub struct RegisteredServer {
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub enabled: bool,
}

/// 一个 MCP 客户端的配置读写适配器
pub trait ClientAdapter {
    /// 稳定的短 id，CLI 的 `--clients` 用这个
    fn id(&self) -> &'static str;
    /// 人读的名字
    fn label(&self) -> &'static str;
    /// 配置文件绝对路径
    fn config_path(&self) -> Result<PathBuf>;
    /// 读出现有服务器列表
    fn list(&self) -> Result<Vec<RegisteredServer>>;
    /// 渲染成该客户端的条目 JSON（`print` 直接输出这个）
    fn render(&self, spec: &ServerSpec) -> Result<serde_json::Value>;
    /// 写入/覆盖一条，返回是否真的产生了改动
    fn upsert(&self, spec: &ServerSpec) -> Result<bool>;
    /// 删除一条，返回是否真的删掉了
    fn remove(&self, name: &str) -> Result<bool>;
    /// 写入之后必须由人做的动作（GUI 点信任、重启等）
    fn post_steps(&self) -> Vec<String>;
}

/// 全部内置客户端，顺序即 `mcp add` 的默认注册顺序
pub fn all_clients() -> Vec<Box<dyn ClientAdapter>> {
    vec![
        Box::new(JsonFileClient::workbuddy()),
        Box::new(JsonFileClient::opencode()),
        Box::new(JsonFileClient::claude_code()),
        Box::new(JsonFileClient::antigravity()),
        Box::new(JsonFileClient::trae()),
    ]
}

/// 按 id 查找客户端；找不到时报出全部可用 id
pub fn find_client(id: &str) -> Result<Box<dyn ClientAdapter>> {
    all_clients()
        .into_iter()
        .find(|c| c.id() == id)
        .ok_or_else(|| {
            let all: Vec<&str> = all_clients().iter().map(|c| c.id()).collect();
            crate::error::McpError::UnknownClient(id.to_string(), all.join(", "))
        })
}

/// 列出全部客户端 id，供提示文案使用
pub fn client_ids() -> Vec<&'static str> {
    all_clients().iter().map(|c| c.id()).collect()
}
