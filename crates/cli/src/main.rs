use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pengj", version, about = "分层模板生成与更新工具")]
struct Cli {
    /// 模板根目录（默认定位 PENGJ_TEMPLATES > 可执行文件旁 templates > 当前目录 templates）
    #[arg(long, global = true)]
    templates: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

/// 解析模板目录，构造 Templates 实例
fn load_templates(templates: &Option<PathBuf>) -> anyhow::Result<pengj_core::Templates> {
    let dir = match templates {
        Some(p) => p.clone(),
        None => pengj_core::default_templates_dir()
            .context("无法自动定位模板目录，请用 --templates 指定")?,
    };
    if !dir.is_dir() {
        anyhow::bail!("模板目录不存在: {}", dir.display());
    }
    Ok(pengj_core::Templates::new(dir))
}

#[derive(Subcommand)]
enum Command {
    /// 列出所有可用层
    ListLayers,
    /// 生成新项目（选层组合输出）
    Create {
        /// 项目名
        name: String,
        /// 选择的层，逗号分隔，如 rust,lefthook
        #[arg(short, long, value_delimiter = ',')]
        layers: Vec<String>,
        /// Rust edition（仅 rust 层生效；允许 2015/2018/2021/2024）
        #[arg(long, default_value = "2021")]
        edition: String,
        /// Rust toolchain channel（仅 rust 层生效；允许 stable/beta/nightly 或版本号如 1.82.0）
        #[arg(long, default_value = "stable")]
        channel: String,
        /// 使用 sccache 编译缓存（默认开；传 --no-sccache 关闭）
        #[arg(long = "no-sccache", action = clap::ArgAction::SetFalse, default_value_t = true)]
        sccache: bool,
        /// 使用 lld 链接器（默认开；传 --no-lld 关闭）
        #[arg(long = "no-lld", action = clap::ArgAction::SetFalse, default_value_t = true)]
        lld: bool,
        /// 中文编程（仅 rust 层生效）：允许中文标识符/字面量，关闭相关命名 lint
        #[arg(long)]
        chinese: bool,
        /// 技能书写语言（仅 agent 层生效）：zh 中文 / en 英文
        #[arg(long, default_value = "zh")]
        skill_lang: String,
        /// 提交信息是否用中文（仅 agent 层生效；默认开，传 --no-commit-zh 关闭）
        #[arg(long = "no-commit-zh", action = clap::ArgAction::SetFalse, default_value_t = true)]
        commit_zh: bool,
        /// 输出父目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// 同步模板更新到已生成的项目
    Update {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let templates = load_templates(&cli.templates)?;
    match cli.command {
        Command::ListLayers => cmd_list_layers(&templates),
        Command::Create {
            name,
            layers,
            edition,
            channel,
            sccache,
            lld,
            chinese,
            skill_lang,
            commit_zh,
            output,
        } => cmd_create(
            &templates,
            &name,
            &layers,
            &edition,
            &channel,
            sccache,
            lld,
            chinese,
            &skill_lang,
            commit_zh,
            &output,
        ),
        Command::Update { dir } => cmd_update(&templates, &dir),
    }
}

fn cmd_list_layers(templates: &pengj_core::Templates) -> anyhow::Result<()> {
    let layers = templates.list_layers().context("读取层列表失败")?;
    if layers.is_empty() {
        println!("(没有可用层)");
        return Ok(());
    }
    println!("{:<10} {:<16} {:<40} 文件数", "ID", "名称", "描述");
    for l in layers {
        let deps = if l.depends.is_empty() {
            "-".to_string()
        } else {
            l.depends.join(",")
        };
        println!(
            "{:<10} {:<16} {:<40} {}  (依赖: {})",
            l.id, l.name, l.description, l.file_count, deps
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_create(
    templates: &pengj_core::Templates,
    name: &str,
    layers: &[String],
    edition: &str,
    channel: &str,
    sccache: bool,
    lld: bool,
    chinese: bool,
    skill_lang: &str,
    commit_zh: bool,
    output: &Path,
) -> anyhow::Result<()> {
    if layers.is_empty() {
        anyhow::bail!("至少需要选择一个层，例如 --layers rust");
    }
    if !["2015", "2018", "2021", "2024"].contains(&edition) {
        anyhow::bail!("无效 edition：{edition}（可选 2015/2018/2021/2024）");
    }
    if channel.is_empty() {
        anyhow::bail!("channel 不能为空（如 stable/nightly/1.82.0）");
    }
    if !["zh", "en"].contains(&skill_lang) {
        anyhow::bail!("无效 skill-lang：{skill_lang}（可选 zh/en）");
    }
    // 构造渲染选项：仅对应层读取，其它层无副作用
    let options: std::collections::BTreeMap<String, serde_json::Value> = [
        ("edition", serde_json::Value::String(edition.to_string())),
        ("channel", serde_json::Value::String(channel.to_string())),
        ("use_sccache", serde_json::Value::Bool(sccache)),
        ("use_lld", serde_json::Value::Bool(lld)),
        ("chinese_programming", serde_json::Value::Bool(chinese)),
        (
            "skill_lang",
            serde_json::Value::String(skill_lang.to_string()),
        ),
        ("commit_zh", serde_json::Value::Bool(commit_zh)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    let report = pengj_core::generate(templates, name, layers, options, output)
        .with_context(|| format!("生成项目 {name} 失败"))?;
    println!("已生成: {}", report.project_dir);
    println!("层顺序: {}", report.layers.join(" -> "));
    println!("文件数: {}", report.files.len());
    for f in &report.files {
        println!("  {f}");
    }
    Ok(())
}

fn cmd_update(templates: &pengj_core::Templates, dir: &Path) -> anyhow::Result<()> {
    let report = pengj_core::update_project(templates, dir).context("更新项目失败")?;
    println!(
        "项目: {}（层: {}）",
        report.project_name,
        report.layers.join(" -> ")
    );
    println!(
        "更新 {} 个文件，新增 {} 个，未变 {} 个",
        report.updated.len(),
        report.created.len(),
        report.unchanged
    );
    for f in &report.updated {
        println!("  [更新] {f}");
    }
    for f in &report.created {
        println!("  [新增] {f}");
    }
    for c in &report.conflicted {
        println!("  [冲突] {} — {}", c.path, c.reason);
    }
    for f in &report.removed {
        println!("  [移除] {} （模板已删除，本地文件保留）", f);
    }
    Ok(())
}
