use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "pengj-templates-cli",
    version,
    about = "分层模板生成与更新工具"
)]
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
    ListLayers {
        /// 以 JSON 数组输出（供脚本消费）
        #[arg(long)]
        json: bool,
    },
    /// 生成新项目（选层组合输出）
    Create {
        /// 项目名
        name: String,
        /// 选择的层，逗号分隔，如 rust,lefthook
        #[arg(short, long, value_delimiter = ',')]
        layers: Vec<String>,
        /// Rust edition（仅 rust 层生效；允许 2015/2018/2021/2024）
        #[arg(long, default_value = "2024")]
        edition: String,
        /// Rust toolchain channel（仅 rust 层生效；允许 stable/beta/nightly 或版本号如 1.82.0）
        #[arg(long, default_value = "stable")]
        channel: String,
        /// 使用 sccache 编译缓存（默认关；传 --sccache 开启）
        #[arg(long, default_value_t = false)]
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
        /// 阶段性完工与收尾时是否要求自动提交并推送（仅 agent 层生效；默认关，传 --commit-and-push 开启）
        #[arg(long = "commit-and-push", default_value_t = false)]
        commit_and_push: bool,
        /// 选择的技能（仅 agent 层生效），逗号分隔如 commit,caveman,grill-me；默认全部
        #[arg(long, value_delimiter = ',')]
        skills: Option<Vec<String>>,
        /// 输出父目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// 同步模板更新到已生成的项目
    Update {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 批量更新多个项目目录（逗号分隔，如 dir1,dir2）
        #[arg(long, value_delimiter = ',')]
        dirs: Option<Vec<PathBuf>>,
        /// 纯受管技能资产与脚本直接对齐覆盖（即使无托管块也不判定为冲突）
        #[arg(long)]
        sync_skills: bool,
    },
    /// 巡检项目的模板对齐状态（只读诊断受管文件与托管块漂移）
    Audit {
        /// 目标项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 批量巡检多个项目目录（逗号分隔，如 dir1,dir2）
        #[arg(long, value_delimiter = ',')]
        dirs: Option<Vec<PathBuf>>,
        /// 显示详细差异对比（diff）
        #[arg(long)]
        diff: bool,
        /// 以 JSON 格式输出（供脚本/自动化消费）
        #[arg(long)]
        json: bool,
    },
    /// 纳管存量已有项目（初始化 .pengj-templates.json 并建立基线）
    Adopt {
        /// 目标项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 选择的层，逗号分隔，如 common,lefthook,agent,rust-workspace
        #[arg(short, long, value_delimiter = ',')]
        layers: Vec<String>,
        /// 强制覆盖已有 manifest
        #[arg(short, long)]
        force: bool,
        /// Rust edition（仅 rust/rust-workspace 层生效；允许 2015/2018/2021/2024）
        #[arg(long, default_value = "2024")]
        edition: String,
        /// Rust toolchain channel（允许 stable/beta/nightly 或版本号）
        #[arg(long, default_value = "stable")]
        channel: String,
        /// 使用 sccache 编译缓存（默认关；传 --sccache 开启）
        #[arg(long, default_value_t = false)]
        sccache: bool,
        /// 使用 lld 链接器（默认开；传 --no-lld 关闭）
        #[arg(long = "no-lld", action = clap::ArgAction::SetFalse, default_value_t = true)]
        lld: bool,
        /// 中文编程：允许中文标识符/字面量，关闭相关命名 lint
        #[arg(long)]
        chinese: bool,
        /// 技能书写语言：zh 中文 / en 英文
        #[arg(long, default_value = "zh")]
        skill_lang: String,
        /// 提交信息是否用中文（默认开，传 --no-commit-zh 关闭）
        #[arg(long = "no-commit-zh", action = clap::ArgAction::SetFalse, default_value_t = true)]
        commit_zh: bool,
        /// 阶段性完工与收尾时是否要求自动提交并推送（默认关，传 --commit-and-push 开启）
        #[arg(long = "commit-and-push", default_value_t = false)]
        commit_and_push: bool,
        /// 选择的技能，逗号分隔如 commit,caveman；默认全部
        #[arg(long, value_delimiter = ',')]
        skills: Option<Vec<String>>,
    },
    /// 安装与配置 MCP 服务器（机器级，跨项目共用）
    Mcp {
        #[command(subcommand)]
        cmd: McpCommand,
    },
}

#[derive(Subcommand, Clone)]
enum McpCommand {
    /// 列出配方目录中的服务器及其在本机的状态
    List {
        /// 以 JSON 数组输出（供脚本消费）
        #[arg(long)]
        json: bool,
    },
    /// 安装主体并注册到客户端全局配置
    Add {
        /// 服务器名（见 `mcp list`）
        name: String,
        /// 目标客户端，逗号分隔；默认全部（workbuddy,opencode,claude-code,agy,trae）
        #[arg(long, value_delimiter = ',')]
        clients: Vec<String>,
        /// 只写配置，跳过主体安装
        #[arg(long)]
        skip_install: bool,
        /// 只打印计划，不落盘
        #[arg(long)]
        dry_run: bool,
        /// 附属产物的源目录（对应仓库副本的根目录）
        #[arg(long)]
        source_root: Option<PathBuf>,
        /// 同时拷贝附属产物（如 RenderDoc UI 扩展）
        #[arg(long)]
        with_side: bool,
        /// 同时改写第三方程序配置以启用（如 RenderDoc AlwaysLoad_Extensions）
        #[arg(long = "enable-app")]
        enable_app: bool,
    },
    /// 从所有客户端配置中移除
    Remove {
        name: String,
        /// 同时卸载主体（uv tool uninstall）
        #[arg(long)]
        purge: bool,
    },
    /// 打印某客户端可粘贴的配置片段
    Print {
        name: String,
        /// 客户端 id（默认 workbuddy）
        #[arg(long)]
        client: Option<String>,
    },
    /// 巡检客户端配置并做真实握手
    Doctor {
        /// 只检查指定服务器
        #[arg(long)]
        server: Option<String>,
        /// 单步握手超时（秒）
        #[arg(long, default_value_t = 8)]
        timeout: u64,
        /// 只做静态检查，不起进程
        #[arg(long = "static-only")]
        static_only: bool,
        /// 就地清理死链条目
        #[arg(long)]
        fix: bool,
        /// 以 JSON 输出（供脚本消费）
        #[arg(long)]
        json: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // mcp 子命令作用于机器上的客户端配置，不依赖模板目录，提前分发
    if let Command::Mcp { cmd } = &cli.command {
        return cmd_mcp(cmd.clone());
    }
    let templates = load_templates(&cli.templates)?;
    match cli.command {
        Command::ListLayers { json } => cmd_list_layers(&templates, json),
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
            commit_and_push,
            skills,
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
            commit_and_push,
            skills.as_deref(),
            &output,
        ),
        Command::Update {
            dir,
            dirs,
            sync_skills,
        } => {
            let targets = match dirs {
                Some(ds) if !ds.is_empty() => ds,
                _ => vec![dir],
            };
            for d in &targets {
                if targets.len() > 1 {
                    println!(
                        "\n==================== 更新项目: {} ====================",
                        d.display()
                    );
                }
                cmd_update(&templates, d, sync_skills)?;
            }
            Ok(())
        }
        Command::Audit {
            dir,
            dirs,
            diff,
            json,
        } => {
            let targets = match dirs {
                Some(ds) if !ds.is_empty() => ds,
                _ => vec![dir],
            };
            let mut any_violations = false;
            for d in &targets {
                if targets.len() > 1 && !json {
                    println!(
                        "\n==================== 巡检项目: {} ====================",
                        d.display()
                    );
                }
                let has_violations = cmd_audit(&templates, d, diff, json)?;
                if has_violations {
                    any_violations = true;
                }
            }
            if any_violations {
                std::process::exit(1);
            }
            Ok(())
        }
        Command::Adopt {
            dir,
            layers,
            force,
            edition,
            channel,
            sccache,
            lld,
            chinese,
            skill_lang,
            commit_zh,
            commit_and_push,
            skills,
        } => cmd_adopt(
            &templates,
            &dir,
            &layers,
            force,
            &edition,
            &channel,
            sccache,
            lld,
            chinese,
            &skill_lang,
            commit_zh,
            commit_and_push,
            skills.as_deref(),
        ),
        Command::Mcp { .. } => unreachable!("mcp 子命令已在加载模板前分发"),
    }
}

fn cmd_list_layers(templates: &pengj_core::Templates, json: bool) -> anyhow::Result<()> {
    let layers = templates.list_layers().context("读取层列表失败")?;
    if json {
        println!("{}", serde_json::to_string_pretty(&layers)?);
        return Ok(());
    }
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
    commit_and_push: bool,
    skills: Option<&[String]>,
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
    let selected_skills: Vec<String> = skills
        .map(|s| s.iter().filter(|x| !x.is_empty()).cloned().collect())
        .unwrap_or_default();
    if !selected_skills.is_empty() {
        let available: Vec<String> = templates
            .list_skills()?
            .into_iter()
            .map(|s| s.name)
            .collect();
        let unknown: Vec<&String> = selected_skills
            .iter()
            .filter(|s| !available.contains(s))
            .collect();
        if !unknown.is_empty() {
            let avail_str = if available.is_empty() {
                "（无）".to_string()
            } else {
                available.join(", ")
            };
            anyhow::bail!(
                "未知技能: {}（可用技能: {}）",
                unknown
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                avail_str
            );
        }
    }
    // 构造渲染选项：仅对应层读取，其它层无副作用
    let mut options: std::collections::BTreeMap<String, serde_json::Value> = [
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
        ("commit_and_push", serde_json::Value::Bool(commit_and_push)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    if !selected_skills.is_empty() {
        options.insert("skills".to_string(), serde_json::json!(selected_skills));
    }
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

fn cmd_update(
    templates: &pengj_core::Templates,
    dir: &Path,
    sync_skills: bool,
) -> anyhow::Result<()> {
    let opts = pengj_core::UpdateOptions { sync_skills };
    let report = pengj_core::update_project_with_options(templates, dir, &opts)
        .with_context(|| format!("更新项目 {} 失败", dir.display()))?;
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
    for f in &report.needs_review {
        println!("  [待复核] {f}");
    }
    for f in &report.removed {
        println!("  [移除] {f}");
    }
    Ok(())
}

fn cmd_audit(
    templates: &pengj_core::Templates,
    dir: &Path,
    show_diff: bool,
    json: bool,
) -> anyhow::Result<bool> {
    let report =
        pengj_core::audit_project(templates, dir, show_diff).context("审计项目模板对齐状态失败")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(report.has_violations);
    }

    println!(
        "项目: {}（层: {}）",
        report.project_name,
        report.layers.join(" -> ")
    );
    println!("------------------------------------------------------------");

    let mut in_sync_count = 0;
    let mut custom_count = 0;
    let mut upstream_newer_count = 0;
    let mut violations = Vec::new();

    for item in &report.items {
        match item.status {
            pengj_core::AuditFileStatus::InSync => in_sync_count += 1,
            pengj_core::AuditFileStatus::ProjectCustomized => {
                custom_count += 1;
                println!("  [项目专属定制] {}", item.path);
            }
            pengj_core::AuditFileStatus::UpstreamNewer => {
                upstream_newer_count += 1;
                println!("  [上游有更新]   {} (可安全运行 update)", item.path);
            }
            pengj_core::AuditFileStatus::ViolatedManagedBlock => {
                violations.push(item);
                println!(
                    "  [托管块被篡改] {} (严重漂移: 受管区间内代码被下游修改)",
                    item.path
                );
            }
            pengj_core::AuditFileStatus::LocallyModifiedFile => {
                violations.push(item);
                println!(
                    "  [受管文件被改] {} (严重漂移: 模板托管文件被下游私自修改)",
                    item.path
                );
            }
            pengj_core::AuditFileStatus::MissingOnDisk => {
                println!("  [本地文件缺失] {}", item.path);
            }
            pengj_core::AuditFileStatus::OrphanInManifest => {
                println!("  [上游已废弃]   {}", item.path);
            }
        }
    }

    println!("------------------------------------------------------------");
    println!(
        "统计: {} 个一致, {} 个项目合规定制, {} 个待更新, {} 个违规漂移",
        in_sync_count,
        custom_count,
        upstream_newer_count,
        violations.len()
    );

    if show_diff && !violations.is_empty() {
        println!("\n================== 详细漂移差异 (Diff) ==================");
        for v in &violations {
            if let Some(d) = &v.diff {
                println!("\n--- 差异: {} ---", v.path);
                print!("{d}");
            }
        }
        println!("=========================================================");
    }

    if report.has_violations {
        eprintln!("\n[警告] 检测到项目存在违规篡改模板内容！请遵循规范：");
        eprintln!("  1. 若改动具备通用价值，应反馈至上游 pengj-templates 模板；");
        eprintln!("  2. 若为项目特化环境，应迁移至托管块外「项目专属区」声明配置；");
        eprintln!("  3. 严禁下游私自保留对模板托管脚本及托管块内的 hack。\n");
    }

    Ok(report.has_violations)
}

#[allow(clippy::too_many_arguments)]
fn cmd_adopt(
    templates: &pengj_core::Templates,
    dir: &Path,
    layers: &[String],
    force: bool,
    edition: &str,
    channel: &str,
    sccache: bool,
    lld: bool,
    chinese: bool,
    skill_lang: &str,
    commit_zh: bool,
    commit_and_push: bool,
    skills: Option<&[String]>,
) -> anyhow::Result<()> {
    if layers.is_empty() {
        anyhow::bail!("必须指定至少一个层，如 --layers common,lefthook,agent");
    }
    let mut selected_skills: Vec<String> = Vec::new();
    if let Some(s_list) = skills {
        let available = templates.list_skills().context("读取可用技能列表失败")?;
        let avail_set: std::collections::HashSet<&str> =
            available.iter().map(|s| s.name.as_str()).collect();
        for s in s_list {
            let s_trimmed = s.trim();
            if s_trimmed.is_empty() {
                continue;
            }
            if avail_set.contains(s_trimmed) {
                selected_skills.push(s_trimmed.to_string());
            } else {
                let avail_str = available
                    .iter()
                    .map(|k| k.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow::bail!("未知的技能: \"{}\"。当前可用技能: {}", s_trimmed, avail_str);
            }
        }
    }
    let mut options: std::collections::BTreeMap<String, serde_json::Value> = [
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
        ("commit_and_push", serde_json::Value::Bool(commit_and_push)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    if !selected_skills.is_empty() {
        options.insert("skills".to_string(), serde_json::json!(selected_skills));
    }

    let report = pengj_core::adopt_project(templates, dir, layers, options, force)
        .context("纳管存量项目失败")?;

    println!("项目已纳管: {}", report.project_name);
    println!("层顺序: {}", report.layers.join(" -> "));
    println!(
        "新增 {} 个模板文件，纳管 {} 个已有文件",
        report.created.len(),
        report.adopted.len()
    );
    for f in &report.created {
        println!("  [新增] {f}");
    }
    for f in &report.adopted {
        println!("  [纳管] {f}");
    }
    for c in &report.conflicted {
        println!("  [冲突] {} — {}", c.path, c.reason);
    }
    for f in &report.needs_review {
        println!("  [待复核] {f}");
    }
    for s in &report.manual_steps {
        println!("  [手动步骤] {s}");
    }
    Ok(())
}

// ---------------- mcp 子命令 ----------------

/// 配方目录：优先 `PENGJ_MCP_CATALOG`，否则叠加 `~/.pengj-mcp/catalog`（存在才生效）
fn load_catalog() -> anyhow::Result<pengj_mcp::Catalog> {
    let dir = std::env::var_os("PENGJ_MCP_CATALOG")
        .map(PathBuf::from)
        .or_else(|| {
            pengj_mcp::paths::config_dir()
                .ok()
                .map(|d| d.join("catalog"))
        });
    pengj_mcp::Catalog::load(dir.as_deref()).context("加载 MCP 配方目录失败")
}

fn cmd_mcp(cmd: McpCommand) -> anyhow::Result<()> {
    let catalog = load_catalog()?;
    match cmd {
        McpCommand::List { json } => cmd_mcp_list(&catalog, json),
        McpCommand::Add {
            name,
            clients,
            skip_install,
            dry_run,
            source_root,
            with_side,
            enable_app,
        } => {
            let opts = pengj_mcp::AddOptions {
                clients,
                skip_install,
                dry_run,
                source_root,
                with_side,
                enable_app,
            };
            cmd_mcp_add(&catalog, &name, &opts)
        }
        McpCommand::Remove { name, purge } => cmd_mcp_remove(&catalog, &name, purge),
        McpCommand::Print { name, client } => cmd_mcp_print(&catalog, &name, client.as_deref()),
        McpCommand::Doctor {
            server,
            timeout,
            static_only,
            fix,
            json,
        } => cmd_mcp_doctor(server, timeout, static_only, fix, json),
    }
}

fn cmd_mcp_list(catalog: &pengj_mcp::Catalog, json: bool) -> anyhow::Result<()> {
    let list = pengj_mcp::status_list(catalog)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&list)?);
        return Ok(());
    }
    if list.is_empty() {
        println!("(配方目录为空)");
        return Ok(());
    }
    for s in list {
        println!(
            "{}  {} · {}",
            s.name,
            if s.installed {
                "主体已就绪"
            } else {
                "主体未安装"
            },
            s.install_kind
        );
        println!("  {}", s.summary);
        println!(
            "  已注册 {} 个客户端: {}",
            s.clients.len(),
            pengj_mcp::join_ids(&s.clients)
        );
    }
    Ok(())
}

fn cmd_mcp_add(
    catalog: &pengj_mcp::Catalog,
    name: &str,
    opts: &pengj_mcp::AddOptions,
) -> anyhow::Result<()> {
    let report = pengj_mcp::add(catalog, name, opts)?;
    println!("配方: {} — {}", report.name, report.summary);
    println!("安装: {}", report.install_state);
    println!("命令: {}", report.command);
    if !report.skipped_clients.is_empty() {
        println!("按配方跳过: {}", report.skipped_clients.join(", "));
    }
    println!("客户端:");
    for w in &report.writes {
        let tag = if report.dry_run {
            "预演"
        } else if w.changed {
            "写入"
        } else {
            "未变"
        };
        println!("  [{tag}] {:<12} {}", w.label, w.path);
    }
    if !report.steps.is_empty() {
        println!("附属动作:");
        for s in &report.steps {
            println!(
                "  [{}] {} — {}",
                if s.ok { "完成" } else { "跳过" },
                s.what,
                s.detail
            );
        }
    }

    let mut post: Vec<String> = Vec::new();
    for w in &report.writes {
        if let Ok(client) = pengj_mcp::find_client(&w.client) {
            for step in client.post_steps() {
                post.push(format!("{}: {step}", client.label()));
            }
        }
    }
    for step in &report.manual_steps {
        post.push(step.clone());
    }
    if !post.is_empty() {
        println!("待人工动作:");
        for (i, step) in post.iter().enumerate() {
            println!("  {}. {step}", i + 1);
        }
    }
    if report.dry_run {
        println!("（预演模式：未落盘、未安装）");
    }
    Ok(())
}

fn cmd_mcp_remove(catalog: &pengj_mcp::Catalog, name: &str, purge: bool) -> anyhow::Result<()> {
    let report = pengj_mcp::remove(catalog, name, purge)?;
    let removed: Vec<_> = report.writes.iter().filter(|w| w.changed).collect();
    if removed.is_empty() {
        println!("{name}: 没有在任何客户端配置里找到");
    } else {
        for w in removed {
            println!("  [移除] {:<12} {}", w.label, w.path);
        }
    }
    if report.purged {
        println!("已卸载主体: {name}");
    }
    if report.cache_removed {
        println!("已清理源码缓存: ~/.pengj-mcp/src/{name}");
    }
    Ok(())
}

fn cmd_mcp_print(
    catalog: &pengj_mcp::Catalog,
    name: &str,
    client: Option<&str>,
) -> anyhow::Result<()> {
    let client_id = client.unwrap_or("workbuddy");
    let adapter = pengj_mcp::find_client(client_id)?;
    let recipe = catalog.require(name)?;
    let command = pengj_mcp::resolve_command(&recipe.run.command)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| recipe.run.command.clone());
    let spec = pengj_mcp::ServerSpec {
        name: recipe.name.clone(),
        command: Some(command),
        args: recipe.run.args.clone(),
        env: recipe.run.env.clone(),
        url: recipe.run.url.clone(),
    };
    let entry = adapter.render(&spec)?;
    println!("# 客户端: {}", adapter.label());
    println!("# 文件: {}", adapter.config_path()?.display());
    println!("{}", serde_json::to_string_pretty(&entry)?);
    Ok(())
}

fn cmd_mcp_doctor(
    server: Option<String>,
    timeout: u64,
    static_only: bool,
    fix: bool,
    json: bool,
) -> anyhow::Result<()> {
    let opts = pengj_mcp::DoctorOptions {
        server,
        timeout: Duration::from_secs(timeout.max(1)),
        static_only,
        fix,
    };
    let report = pengj_mcp::doctor(&opts)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        if report.problems() > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    if report.checks.is_empty() {
        println!("(没有找到任何 MCP 配置条目)");
    } else {
        println!("== 客户端配置 ==");
        for c in &report.checks {
            let name = if c.name == "-" {
                "(整份配置)"
            } else {
                c.name.as_str()
            };
            println!(
                "  [{}] {:<12} {:<14} {}",
                c.status.label(),
                c.client,
                name,
                c.detail
            );
        }
    }
    if !report.fixed.is_empty() {
        println!("已清理死链: {}", report.fixed.join(", "));
    }
    if !report.handshakes.is_empty() {
        println!("== 握手校验 ==");
        for h in &report.handshakes {
            if h.ok {
                let version = h.server_version.clone().unwrap_or_default();
                println!(
                    "  [正常] {} — {} {} · {} 个工具 · {}ms",
                    h.target,
                    h.server_name.as_deref().unwrap_or("?"),
                    version,
                    h.tools,
                    h.elapsed_ms
                );
                if let Some(note) = &h.note {
                    println!("         注意: {note}");
                }
            } else {
                println!(
                    "  [失败] {} — {}",
                    h.target,
                    h.note.as_deref().unwrap_or("未知错误")
                );
            }
        }
    }

    let problems = report.problems();
    println!(
        "统计: {} 条配置, {} 个握手, {} 个问题",
        report.checks.len(),
        report.handshakes.len(),
        problems
    );
    if problems > 0 {
        std::process::exit(1);
    }
    Ok(())
}
