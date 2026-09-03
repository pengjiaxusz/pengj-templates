//! 托管块（managed block）识别与替换
//!
//! 模板文件里用成对的注释标记圈定「受托管区间」：区间内的内容随模板更新，
//! 区间外的用户自定义内容原样保留。支持三种注释风格：
//!
//! - HTML/Markdown：`<!-- PENGJ_TEMPLATE_START -->` … `<!-- PENGJ_TEMPLATE_END -->`
//! - Hash（TOML/YAML/Shell/Python）：`# PENGJ_TEMPLATE_START` … `# PENGJ_TEMPLATE_END`
//! - Slash（Rust/JS/TS/C/C++）：`// PENGJ_TEMPLATE_START` … `// PENGJ_TEMPLATE_END`

/// 托管块的注释标记风格
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum BlockStyle {
    /// HTML/Markdown 注释：`<!-- PENGJ_TEMPLATE_START -->` … `<!-- PENGJ_TEMPLATE_END -->`
    Html,
    /// Hash 注释（TOML/YAML/Shell/Python）：`# PENGJ_TEMPLATE_START` … `# PENGJ_TEMPLATE_END`
    Hash,
    /// Slash 注释（Rust/JS/TS/C/C++）：`// PENGJ_TEMPLATE_START` … `// PENGJ_TEMPLATE_END`
    Slash,
}

impl BlockStyle {
    /// 起始标记
    pub fn start_marker(self) -> &'static str {
        match self {
            BlockStyle::Html => "<!-- PENGJ_TEMPLATE_START -->",
            BlockStyle::Hash => "# PENGJ_TEMPLATE_START",
            BlockStyle::Slash => "// PENGJ_TEMPLATE_START",
        }
    }

    /// 结束标记
    pub fn end_marker(self) -> &'static str {
        match self {
            BlockStyle::Html => "<!-- PENGJ_TEMPLATE_END -->",
            BlockStyle::Hash => "# PENGJ_TEMPLATE_END",
            BlockStyle::Slash => "// PENGJ_TEMPLATE_END",
        }
    }
}

/// 从文本中识别出的托管块
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ManagedBlock {
    /// 注释风格
    pub style: BlockStyle,
    /// 起始标记开头的字节偏移（含）
    pub start: usize,
    /// 结束标记之后的字节偏移（不含）
    pub end: usize,
    /// 起始标记与结束标记之间的正文（不含两个标记本身）
    pub body: String,
    /// 从 `start` 到 `end` 的完整区间原文（含两个标记）
    pub full: String,
}

/// 从文本中识别第一个完整的托管块（起始标记与结束标记均存在）。
///
/// 按 [`BlockStyle`] 的声明顺序查找，命中某种风格的完整配对即返回。
/// 只出现 START 或只出现 END 等不完整标记时返回 `None`，不会 panic。
pub fn extract_managed_block(text: &str) -> Option<ManagedBlock> {
    const STYLES: [BlockStyle; 3] = [BlockStyle::Html, BlockStyle::Hash, BlockStyle::Slash];

    for style in STYLES {
        if let Some(block) = extract_block_of_style(text, style) {
            return Some(block);
        }
    }
    None
}

/// 按指定风格在文本中查找托管块：起始标记在前、结束标记在其后
fn extract_block_of_style(text: &str, style: BlockStyle) -> Option<ManagedBlock> {
    let start_marker = style.start_marker();
    let end_marker = style.end_marker();

    let start = text.find(start_marker)?;
    let after_start = start + start_marker.len();
    let end_rel = text[after_start..].find(end_marker)?;
    let end = after_start + end_rel + end_marker.len();

    Some(ManagedBlock {
        style,
        start,
        end,
        body: text[after_start..after_start + end_rel].to_string(),
        full: text[start..end].to_string(),
    })
}

/// 托管块在目标文件缺失时的放置策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockPlacement {
    /// 追加到末尾（默认）
    #[default]
    Append,
    /// 置顶插入到开头（适用于 AGENTS.md 等规范文件，保证框架总则处于最前）
    Prepend,
}

/// 用 `incoming_text` 中的托管块更新 `target_text` 中的同风格托管块。
///
/// 规则：
/// - `incoming_text` 不含托管块：原样返回 `incoming_text`。
/// - `target_text` 含同风格托管块：仅替换该区间（含两个标记），区间外的内容逐字节保留。
/// - 其余情况（target 无托管块，或风格不匹配）：把 incoming 的托管块追加到
///   `target_text` 末尾，中间用一个空行分隔，并保证结果以换行结尾。
pub fn replace_managed_block(target_text: &str, incoming_text: &str) -> String {
    replace_managed_block_with_placement(target_text, incoming_text, BlockPlacement::Append)
}

/// 支持指定放置策略（Append / Prepend）的托管块替换
pub fn replace_managed_block_with_placement(
    target_text: &str,
    incoming_text: &str,
    placement: BlockPlacement,
) -> String {
    let Some(incoming) = extract_managed_block(incoming_text) else {
        return incoming_text.to_string();
    };

    match extract_managed_block(target_text) {
        Some(target) if target.style == incoming.style => {
            let mut out = String::with_capacity(target_text.len() + incoming.full.len());
            out.push_str(&target_text[..target.start]);
            out.push_str(&incoming.full);
            out.push_str(&target_text[target.end..]);
            out
        }
        _ => match placement {
            BlockPlacement::Append => append_block(target_text, &incoming.full),
            BlockPlacement::Prepend => prepend_block(target_text, &incoming.full),
        },
    }
}

/// 把托管块置顶插入到目标文本开头：空行分隔，保证尾随换行
fn prepend_block(target_text: &str, block: &str) -> String {
    if target_text.is_empty() {
        let mut out = String::with_capacity(block.len() + 1);
        out.push_str(block);
        ensure_trailing_newline(&mut out);
        return out;
    }

    let mut out = String::with_capacity(target_text.len() + block.len() + 3);
    out.push_str(block.trim_end());
    out.push_str("\n\n");
    out.push_str(target_text.trim_start());
    ensure_trailing_newline(&mut out);
    out
}

/// 把托管块追加到目标文本末尾：空行分隔，保证尾随换行
fn append_block(target_text: &str, block: &str) -> String {
    if target_text.is_empty() {
        let mut out = String::with_capacity(block.len() + 1);
        out.push_str(block);
        ensure_trailing_newline(&mut out);
        return out;
    }

    let mut out = String::with_capacity(target_text.len() + block.len() + 3);
    out.push_str(target_text);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
    out.push_str(block);
    ensure_trailing_newline(&mut out);
    out
}

/// 保证字符串以单个换行结尾
fn ensure_trailing_newline(out: &mut String) {
    if !out.ends_with('\n') {
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HTML_BLOCK: &str =
        "<!-- PENGJ_TEMPLATE_START -->\nmanaged\n<!-- PENGJ_TEMPLATE_END -->\n";
    const HASH_BLOCK: &str = "# PENGJ_TEMPLATE_START\nmanaged = true\n# PENGJ_TEMPLATE_END\n";
    const SLASH_BLOCK: &str = "// PENGJ_TEMPLATE_START\nmanaged();\n// PENGJ_TEMPLATE_END\n";

    // ---------- extract_managed_block ----------

    #[test]
    fn extract_html_markdown_block() {
        let text = format!("# Header\n\n{HTML_BLOCK}\n\nfooter");
        let block = extract_managed_block(&text).expect("应识别 HTML 托管块");
        assert_eq!(block.style, BlockStyle::Html);
        assert_eq!(block.body, "\nmanaged\n");
        // full 为 [start, end) 区间原文，不含结束标记后的换行
        assert_eq!(
            block.full,
            "<!-- PENGJ_TEMPLATE_START -->\nmanaged\n<!-- PENGJ_TEMPLATE_END -->"
        );
        // 区间含两个标记本身，与源文本切片一致
        assert_eq!(&text[block.start..block.end], block.full);
    }

    #[test]
    fn extract_hash_block() {
        let block = extract_managed_block(HASH_BLOCK).expect("应识别 Hash 托管块");
        assert_eq!(block.style, BlockStyle::Hash);
        assert_eq!(block.body, "\nmanaged = true\n");
        assert_eq!(
            block.full,
            "# PENGJ_TEMPLATE_START\nmanaged = true\n# PENGJ_TEMPLATE_END"
        );
        assert_eq!(&HASH_BLOCK[block.start..block.end], block.full);
    }

    #[test]
    fn extract_slash_block() {
        let block = extract_managed_block(SLASH_BLOCK).expect("应识别 Slash 托管块");
        assert_eq!(block.style, BlockStyle::Slash);
        assert_eq!(block.body, "\nmanaged();\n");
        assert_eq!(
            block.full,
            "// PENGJ_TEMPLATE_START\nmanaged();\n// PENGJ_TEMPLATE_END"
        );
        assert_eq!(&SLASH_BLOCK[block.start..block.end], block.full);
    }

    #[test]
    fn extract_whitespace_around_markers() {
        let text =
            "\n  <!-- PENGJ_TEMPLATE_START -->\n  managed\n  <!-- PENGJ_TEMPLATE_END -->  \n";
        let block = extract_managed_block(text).expect("带缩进/尾随空白的标记应被识别");
        assert_eq!(block.style, BlockStyle::Html);
        assert_eq!(block.body, "\n  managed\n  ");
        // start 指向标记本身，不含行首空白；end 在结束标记之后，不含行尾空白
        assert_eq!(
            &text[block.start..block.start + 29],
            "<!-- PENGJ_TEMPLATE_START -->"
        );
        assert_eq!(
            &text[block.end - 27..block.end],
            "<!-- PENGJ_TEMPLATE_END -->"
        );
    }

    #[test]
    fn extract_prefers_first_complete_pair() {
        // 第一个 START 后紧跟 END，第二个 START 没有配对
        let text = "<!-- PENGJ_TEMPLATE_START -->\nA\n<!-- PENGJ_TEMPLATE_END -->\n\n<!-- PENGJ_TEMPLATE_START -->\nB\n";
        let block = extract_managed_block(text).expect("应取第一个完整配对");
        assert_eq!(block.body, "\nA\n");
        assert_eq!(
            &text[block.start..block.end],
            "<!-- PENGJ_TEMPLATE_START -->\nA\n<!-- PENGJ_TEMPLATE_END -->"
        );
    }

    #[test]
    fn extract_none_when_no_markers() {
        assert_eq!(extract_managed_block("plain text without markers"), None);
        assert_eq!(extract_managed_block(""), None);
    }

    #[test]
    fn extract_none_on_incomplete_markers() {
        // 只有 START 没有 END
        assert_eq!(
            extract_managed_block("<!-- PENGJ_TEMPLATE_START -->\nnever closed"),
            None
        );
        // 只有 END 没有 START
        assert_eq!(
            extract_managed_block("# PENGJ_TEMPLATE_END\nno start"),
            None
        );
        // END 出现在 START 之前，不算配对
        assert_eq!(
            extract_managed_block("// PENGJ_TEMPLATE_END\n// PENGJ_TEMPLATE_START\n"),
            None
        );
    }

    // ---------- replace_managed_block ----------

    #[test]
    fn replace_markdown_block_at_beginning() {
        let target = format!("{HTML_BLOCK}\n## Custom Rules\n- keep me\n");
        let incoming = "<!-- PENGJ_TEMPLATE_START -->\nnew managed\n<!-- PENGJ_TEMPLATE_END -->\n";
        let merged = replace_managed_block(&target, incoming);
        assert_eq!(
            merged,
            "<!-- PENGJ_TEMPLATE_START -->\nnew managed\n<!-- PENGJ_TEMPLATE_END -->\n\n## Custom Rules\n- keep me\n"
        );
    }

    #[test]
    fn replace_markdown_block_in_middle() {
        let target = "# Header\n\n<!-- PENGJ_TEMPLATE_START -->\nold template\n<!-- PENGJ_TEMPLATE_END -->\n\n## Custom User Rules\n- Rule 1\n";
        let incoming =
            "<!-- PENGJ_TEMPLATE_START -->\nnew updated template\n<!-- PENGJ_TEMPLATE_END -->\n";
        let merged = replace_managed_block(target, incoming);
        assert!(merged.starts_with("# Header\n\n"));
        assert!(merged.contains("new updated template"));
        assert!(merged.ends_with("\n\n## Custom User Rules\n- Rule 1\n"));
        assert!(!merged.contains("old template"));
    }

    #[test]
    fn replace_markdown_block_at_end() {
        let target =
            "# Header\n\n<!-- PENGJ_TEMPLATE_START -->\nold\n<!-- PENGJ_TEMPLATE_END -->\n";
        let incoming = "<!-- PENGJ_TEMPLATE_START -->\nnew\n<!-- PENGJ_TEMPLATE_END -->\n";
        let merged = replace_managed_block(target, incoming);
        assert_eq!(
            merged,
            "# Header\n\n<!-- PENGJ_TEMPLATE_START -->\nnew\n<!-- PENGJ_TEMPLATE_END -->\n"
        );
    }

    #[test]
    fn replace_hash_block_preserves_custom_cargo_config() {
        // 模拟 .cargo/config.toml：块外的 [build] rustflags 与 [alias] 必须逐字节保留
        let target = "[build]\nrustflags = [\"-C\", \"target-cpu=native\"]\n\n# PENGJ_TEMPLATE_START\nmanaged = \"old\"\n# PENGJ_TEMPLATE_END\n\n[alias]\nb = \"build\"\n";
        let incoming =
            "# PENGJ_TEMPLATE_START\nmanaged = \"new\"\nextra = 1\n# PENGJ_TEMPLATE_END\n";
        let merged = replace_managed_block(target, incoming);
        assert_eq!(
            merged,
            "[build]\nrustflags = [\"-C\", \"target-cpu=native\"]\n\n# PENGJ_TEMPLATE_START\nmanaged = \"new\"\nextra = 1\n# PENGJ_TEMPLATE_END\n\n[alias]\nb = \"build\"\n"
        );
    }

    #[test]
    fn replace_slash_block_preserves_surroundings() {
        let target = "fn main() {\n    // PENGJ_TEMPLATE_START\n    old_code();\n    // PENGJ_TEMPLATE_END\n    user_code();\n}\n";
        let incoming = "// PENGJ_TEMPLATE_START\n    new_code();\n// PENGJ_TEMPLATE_END\n";
        let merged = replace_managed_block(target, incoming);
        assert_eq!(
            merged,
            "fn main() {\n    // PENGJ_TEMPLATE_START\n    new_code();\n// PENGJ_TEMPLATE_END\n    user_code();\n}\n"
        );
    }

    #[test]
    fn replace_preserves_bytes_outside_block() {
        let target = "前<!-- PENGJ_TEMPLATE_START -->中<!-- PENGJ_TEMPLATE_END -->后";
        let incoming = "<!-- PENGJ_TEMPLATE_START -->新<!-- PENGJ_TEMPLATE_END -->";
        let merged = replace_managed_block(target, incoming);
        assert_eq!(
            merged,
            "前<!-- PENGJ_TEMPLATE_START -->新<!-- PENGJ_TEMPLATE_END -->后"
        );
    }

    #[test]
    fn replace_with_whitespace_around_target_markers() {
        // 标记周围有空白：区间外空白逐字节保留（行首缩进在 START 前、行尾空白在 END 后），
        // 区间内（标记 + 正文）整体替换为 incoming 内容
        let target = "# Header\n\n  <!-- PENGJ_TEMPLATE_START -->  \nold\n  <!-- PENGJ_TEMPLATE_END -->  \n\nfooter\n";
        let incoming = "<!-- PENGJ_TEMPLATE_START -->\nnew\n<!-- PENGJ_TEMPLATE_END -->\n";
        let merged = replace_managed_block(target, incoming);
        assert_eq!(
            merged,
            "# Header\n\n  <!-- PENGJ_TEMPLATE_START -->\nnew\n<!-- PENGJ_TEMPLATE_END -->  \n\nfooter\n"
        );
    }

    #[test]
    fn append_block_when_target_has_no_markers() {
        let target = "# Header\n\n[build]\nrustflags = [\"-C\", \"target-cpu=native\"]\n\n[alias]\nb = \"build\"\n";
        let incoming = HASH_BLOCK;
        let merged = replace_managed_block(target, incoming);
        // 空行分隔，块内内容完整，尾部有换行
        assert_eq!(
            merged,
            "# Header\n\n[build]\nrustflags = [\"-C\", \"target-cpu=native\"]\n\n[alias]\nb = \"build\"\n\n# PENGJ_TEMPLATE_START\nmanaged = true\n# PENGJ_TEMPLATE_END\n"
        );
    }

    #[test]
    fn append_block_normalizes_trailing_newline() {
        // target 没有尾随换行：先补换行再补空行
        let target = "# Header\nNo trailing newline";
        let incoming = HASH_BLOCK;
        let merged = replace_managed_block(target, incoming);
        assert_eq!(
            merged,
            "# Header\nNo trailing newline\n\n# PENGJ_TEMPLATE_START\nmanaged = true\n# PENGJ_TEMPLATE_END\n"
        );
    }

    #[test]
    fn append_block_reuses_existing_trailing_blank_line() {
        // target 已以空行结尾：不再额外加空行
        let target = "# Header\n\n";
        let merged = replace_managed_block(target, HASH_BLOCK);
        assert_eq!(
            merged,
            "# Header\n\n# PENGJ_TEMPLATE_START\nmanaged = true\n# PENGJ_TEMPLATE_END\n"
        );
    }

    #[test]
    fn append_block_to_empty_target() {
        let merged = replace_managed_block("", HASH_BLOCK);
        assert_eq!(merged, HASH_BLOCK);
    }

    #[test]
    fn append_when_target_style_differs() {
        // target 里是 Hash 块，incoming 是 Slash 块：无同风格匹配，追加
        let target = "# PENGJ_TEMPLATE_START\nowned\n# PENGJ_TEMPLATE_END\n";
        let incoming = SLASH_BLOCK;
        let merged = replace_managed_block(target, incoming);
        assert_eq!(
            merged,
            "# PENGJ_TEMPLATE_START\nowned\n# PENGJ_TEMPLATE_END\n\n// PENGJ_TEMPLATE_START\nmanaged();\n// PENGJ_TEMPLATE_END\n"
        );
    }

    #[test]
    fn incoming_without_block_returns_incoming() {
        let target = "# Header\n<!-- PENGJ_TEMPLATE_START -->\nold\n<!-- PENGJ_TEMPLATE_END -->\n";
        let incoming = "no markers here";
        let merged = replace_managed_block(target, incoming);
        assert_eq!(merged, "no markers here");
    }

    #[test]
    fn incoming_incomplete_block_returns_incoming() {
        let target = "# Header\n<!-- PENGJ_TEMPLATE_START -->\nold\n<!-- PENGJ_TEMPLATE_END -->\n";
        let incoming = "<!-- PENGJ_TEMPLATE_START -->\nnever closed";
        let merged = replace_managed_block(target, incoming);
        assert_eq!(merged, "<!-- PENGJ_TEMPLATE_START -->\nnever closed");
    }

    #[test]
    fn prepend_block_when_target_has_no_markers() {
        let target = "# Custom User Rules\n- Rule 1\n- Rule 2\n";
        let merged =
            replace_managed_block_with_placement(target, HTML_BLOCK, BlockPlacement::Prepend);
        assert_eq!(
            merged,
            "<!-- PENGJ_TEMPLATE_START -->\nmanaged\n<!-- PENGJ_TEMPLATE_END -->\n\n# Custom User Rules\n- Rule 1\n- Rule 2\n"
        );
    }

    #[test]
    fn prepend_block_to_empty_target() {
        let merged = replace_managed_block_with_placement("", HTML_BLOCK, BlockPlacement::Prepend);
        assert_eq!(merged, HTML_BLOCK);
    }

    #[test]
    fn prepend_block_replaces_in_place_when_target_has_block() {
        let target = "<!-- PENGJ_TEMPLATE_START -->\nold\n<!-- PENGJ_TEMPLATE_END -->\n\n# Custom User Rules\n";
        let merged =
            replace_managed_block_with_placement(target, HTML_BLOCK, BlockPlacement::Prepend);
        assert_eq!(
            merged,
            "<!-- PENGJ_TEMPLATE_START -->\nmanaged\n<!-- PENGJ_TEMPLATE_END -->\n\n# Custom User Rules\n"
        );
    }
}
