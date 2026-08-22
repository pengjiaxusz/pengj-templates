use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("未知的层: {0}")]
    UnknownLayer(String),

    #[error("层存在循环依赖: {0:?}")]
    CircularDependency(Vec<String>),

    #[error("项目名无效: {0}")]
    InvalidProjectName(String),

    #[error("目标目录已存在且非空: {0}")]
    DirExists(String),

    #[error("未找到 .pengj.json manifest，目录可能不是由 pengj-templates 生成: {0}")]
    MissingManifest(String),

    #[error("模板渲染失败: {0}")]
    Render(String),

    #[error("模板目录不存在: {0}")]
    TemplateRoot(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML 解析错误: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
