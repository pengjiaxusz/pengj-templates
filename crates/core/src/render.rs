use minijinja::Environment;

use crate::context::RenderContext;
use crate::error::{CoreError, Result};

/// 渲染一段模板文本
pub fn render_text(template: &str, ctx: &RenderContext) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("t", template)
        .map_err(|e| CoreError::Render(e.to_string()))?;
    env.get_template("t")
        .unwrap()
        .render(minijinja::context! {
            project_name => ctx.project_name,
            project_slug => ctx.project_slug,
            year => ctx.year,
            layers => ctx.layers,
            options => ctx.options,
        })
        .map_err(|e| CoreError::Render(e.to_string()))
}

/// 粗略判断是否为文本文件（前 8KB 内无 NUL 字节即视为文本）
pub fn is_text(bytes: &[u8]) -> bool {
    let probe = &bytes[..bytes.len().min(8192)];
    !probe.contains(&0)
}
