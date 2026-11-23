use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};

mod self_update;
mod update;

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
    /// 跳过 CLI 自身的更新检查（等效于设置环境变量 PENGJ_NO_UPDATE_CHECK=1）
    #[arg(long = "no-update-check", global = true)]
    no_update_check: bool,
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
        /// 选择的层，逗号分隔，如 rust,gate
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
        /// 项目文案语言：zh 中文 / en 英文（技能文档及各层注释、报错、日志）
        #[arg(long, default_value = "zh")]
        skill_lang: String,
        /// 提交信息是否用中文（决定门禁的提交主题语言规则与提交规范文档；默认开，传 --no-commit-zh 关闭）
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
    /// 给已纳管项目增量加层（合并层清单后按新层重渲染，不做全量重纳管）
    AddLayer {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 要追加的层，逗号分隔，如 just,bque
        #[arg(short, long, value_delimiter = ',')]
        layers: Vec<String>,
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
    /// 校验已生成项目的文档语言：en 项目零汉字，zh 项目文案必须含中文
    ///
    /// 按项目 manifest 的 skill_lang 判定，可用 --lang 覆盖；只检查模板托管文件。
    /// 与 `audit`（查模板对齐漂移）互补，专查「文案没跟着语言轴走」这类基础错误。
    CheckLang {
        /// 目标项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 覆盖语言判定：zh / en
        #[arg(long)]
        lang: Option<String>,
        /// 以 JSON 格式输出（供脚本/自动化消费）
        #[arg(long)]
        json: bool,
    },
    /// 纳管存量已有项目（初始化 .pengj-templates/manifest.json 并建立基线）
    Adopt {
        /// 目标项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 选择的层，逗号分隔，如 common,gate,agent,rust-workspace
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
        /// 项目文案语言：zh 中文 / en 英文（技能文档及各层注释、报错、日志）
        #[arg(long, default_value = "zh")]
        skill_lang: String,
        /// 提交信息是否用中文（决定门禁的提交主题语言规则与提交规范文档；默认开，传 --no-commit-zh 关闭）
        #[arg(long = "no-commit-zh", action = clap::ArgAction::SetFalse, default_value_t = true)]
        commit_zh: bool,
        /// 阶段性完工与收尾时是否要求自动提交并推送（默认关，传 --commit-and-push 开启）
        #[arg(long = "commit-and-push", default_value_t = false)]
        commit_and_push: bool,
        /// 选择的技能，逗号分隔如 commit,caveman；默认全部
        #[arg(long, value_delimiter = ',')]
        skills: Option<Vec<String>>,
    },
    /// 仓库级初始化：工作区链接、仓库级 Git 配置与子模块补缺
    ///
    /// 不依赖模板目录；供 `just doctor --fix` 调用，也可单独使用。
    Repo {
        #[command(subcommand)]
        cmd: RepoCommand,
    },
    /// 安装与配置 MCP 服务器（机器级，跨项目共用）
    Mcp {
        #[command(subcommand)]
        cmd: McpCommand,
    },
    /// 提交门禁：约定式提交校验、代码语言守卫与 git 钩子安装
    ///
    /// 门禁内置于本 CLI，无独立二进制；参数原样透传：
    /// `audit`（默认）、`fmt`、`commit-msg <文件>`、`hook pre-commit|commit-msg|pre-push`、
    /// `hooks install`、`selftest`。
    Gate {
        /// 透传给门禁的参数
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// 子仓库：读取机器可读的登记表，渲染清单或做健康体检
    ///
    /// 登记表是子仓库的唯一事实源，与子仓库同步技能读的是同一份文件。
    Subrepo {
        #[command(subcommand)]
        cmd: SubrepoCommand,
    },
    /// 工作树：共享编译目录的推导与校验、按名新建工作树
    ///
    /// 共享目录就是主工作树的 `target/`（取 `git worktree list` 第一条），
    /// 同一仓库的所有工作树推出同一个值，构建配方与自检读同一个值。
    Worktree {
        #[command(subcommand)]
        cmd: WorktreeCommand,
    },
    /// 项目环境自检：按 manifest 启用的层检查工具链、提交钩子、工作区链接、子仓库与共享编译目录
    ///
    /// 不依赖模板目录；以 `.pengj-templates/manifest.json` 为唯一事实源。
    /// 退出码：0 无错误（允许存在警告），1 存在错误。
    Doctor {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 本地修复模式：激活提交钩子、补齐工作区链接与共享编译目录（永不联网）
        #[arg(long)]
        fix: bool,
        /// 覆盖输出语言：zh / en（默认取 manifest.options.skill_lang，缺省 zh）
        #[arg(long)]
        lang: Option<String>,
        /// 以 JSON 格式输出结构化报告（键名恒为英文）
        #[arg(long)]
        json: bool,
    },
    /// 一键装机：按 manifest 启用的层安装缺失工具链、拉齐子模块并完成本地初始化
    ///
    /// 不依赖模板目录；以 `.pengj-templates/manifest.json` 为唯一事实源。
    /// 退出码：0 全部就绪或仅含告警，1 存在失败步骤。
    Setup {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 仅计算并打印装机计划，不执行任何下载、安装或写入
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// 强制按临时受限环境执行（自动关闭 Rust 增量编译）
        #[arg(long, alias = "ci", conflicts_with = "no_ephemeral")]
        ephemeral: bool,
        /// 禁止临时受限环境优化（覆盖 CI 与磁盘空间自动探测）
        #[arg(long = "no-ephemeral", conflicts_with = "ephemeral")]
        no_ephemeral: bool,
        /// 覆盖输出语言：zh / en（默认取 manifest.options.skill_lang，缺省 zh）
        #[arg(long)]
        lang: Option<String>,
        /// 以 JSON 格式输出结构化报告（键名恒为英文）
        #[arg(long)]
        json: bool,
    },
    /// 立即检查 CLI 是否有新版本（同步联网，不写提示状态之外的任何东西）
    ///
    /// 日常运行时由后台异步检查代替，这条命令用于脚本或想立刻知道答案的场合。
    CheckUpdate {
        /// 以 JSON 格式输出（供脚本/自动化消费）
        #[arg(long)]
        json: bool,
        /// 联网超时（秒，默认 8）
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// 后台更新检查本体（内部使用，不由用户直接调用）
    #[command(name = "__update-check", hide = true)]
    BackgroundUpdateCheck,
    /// 更新本工具自身（二进制 + 运行时模板）
    ///
    /// 与 `update` 是两层事：`update` 把模板同步进项目，`self update` 换的是
    /// 装在机器上的这个程序。默认取 GitHub Releases 的最新版，`--from-source`
    /// 用当前工作区的构建产物（拿得到未发布的改动）。
    #[command(name = "self")]
    SelfCmd {
        #[command(subcommand)]
        cmd: SelfCommand,
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

#[derive(Subcommand, Clone)]
enum SubrepoCommand {
    /// 把登记表渲染成人类可读的清单（名称、相对路径、依赖模式、上游跟踪分支）
    ///
    /// 登记表为空或条目缺少必填字段时直接报错退出，不输出半成品。
    List {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 以 JSON 格式输出（供脚本/自动化消费）
        #[arg(long)]
        json: bool,
    },
    /// 对项目做子仓库体检：嵌套工作区红线、子模块就绪状态、依赖模式自洽
    ///
    /// 退出码分级：0 全部通过，1 有失败项，2 只有警告项。
    Check {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 以 JSON 格式输出（供脚本/自动化消费）
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Clone)]
enum WorktreeCommand {
    /// 打印共享编译目录的绝对路径（构建配方与自检都读这个值，不各自推导）
    TargetDir {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 目录缺失时顺手创建（只补缺、不覆盖）
        #[arg(long)]
        ensure: bool,
        /// 以 JSON 格式输出（供脚本消费）
        #[arg(long)]
        json: bool,
    },
    /// 校验共享编译目录：存在、可写、没有指向别处的覆盖值
    ///
    /// 退出码：0 就绪，1 缺失或不可写，2 就绪但被环境变量指向了别处。
    Check {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 修复模式：创建缺失的目录（只补缺、不覆盖）
        #[arg(long)]
        fix: bool,
        /// 以 JSON 格式输出（供脚本消费）
        #[arg(long)]
        json: bool,
    },
    /// 新建工作树：分支 + 目录一次做完，接线共享编译目录并补齐子模块
    ///
    /// 目录建在主仓同级、以主仓目录名为前缀；目标已存在时明确报错。
    Create {
        /// 工作树名称（同时作为分支名与目录后缀）
        name: String,
        /// 主仓目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 以 JSON 格式输出（供脚本消费）
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Clone)]
enum RepoCommand {
    /// 检查工作区链接与仓库级配置（缺失或冲突时退出码非 0）
    Status {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 追加或覆盖链接，格式 <link>=<target>（相对项目根，可重复）
        #[arg(long = "link", value_name = "LINK=TARGET")]
        links: Vec<String>,
        /// 以 JSON 格式输出（供脚本/自动化消费）
        #[arg(long)]
        json: bool,
    },
    /// 创建缺失的工作区链接并补齐仓库级配置（幂等，不覆盖已有内容）
    Init {
        /// 项目目录（默认当前目录）
        #[arg(short, long, default_value = ".")]
        dir: PathBuf,
        /// 追加或覆盖链接，格式 <link>=<target>（相对项目根，可重复）
        #[arg(long = "link", value_name = "LINK=TARGET")]
        links: Vec<String>,
        /// 跳过子模块补缺（默认在存在 .gitmodules 时只初始化未初始化的子模块）
        #[arg(long)]
        no_submodules: bool,
        /// 以 JSON 格式输出（供脚本/自动化消费）
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Clone)]
enum SelfCommand {
    /// 升级安装根下的 CLI 与模板（默认最新发布版；--from-source 用本机构建）
    Update {
        /// 用当前工作区的源码构建后安装，而不是下载发布包
        #[arg(long = "from-source")]
        from_source: bool,
        /// 指定版本（回滚用；缺省取最新发布版）
        #[arg(long, value_name = "VERSION")]
        version: Option<String>,
        /// 同版本也重装、允许降级
        #[arg(long)]
        force: bool,
        /// 只打印计划，不落盘
        #[arg(long = "dry-run")]
        dry_run: bool,
        /// 不刷新 PATH 上的其它副本
        #[arg(long = "no-path-refresh")]
        no_path_refresh: bool,
        /// --from-source 的工作区根（缺省从当前目录向上找）
        #[arg(long, value_name = "DIR")]
        workspace: Option<PathBuf>,
        /// 查询版本的联网超时（秒，默认 8）
        #[arg(long, value_name = "SECONDS")]
        timeout: Option<u64>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // 后台更新检查本体：先于一切分发处理，不加载模板、也不会再触发一次检查
    if matches!(cli.command, Command::BackgroundUpdateCheck) {
        std::process::exit(update::run_background());
    }
    // 更新检查只影响「提示」这一层，不改变任何命令的行为。
    // gate 跑在 git 钩子里，要求绝对安静、绝对快，一律跳过；
    // check-update 自己就会同步联网，不必再派生后台检查。
    let skip = cli.no_update_check
        || matches!(
            cli.command,
            Command::Gate { .. } | Command::CheckUpdate { .. } | Command::SelfCmd { .. }
        );
    update::begin(skip);
    let result = run(cli);
    update::finish();
    result
}

fn run(cli: Cli) -> anyhow::Result<()> {
    // check-update 只查版本，不依赖模板目录，提前分发
    if let Command::CheckUpdate { json, timeout } = &cli.command {
        let options = update::CheckOptions {
            json: *json,
            timeout: *timeout,
        };
        update::run_check(&options)?;
        return Ok(());
    }
    // repo 子命令只作用于项目目录，不依赖模板目录，提前分发
    if let Command::Repo { cmd } = &cli.command {
        return cmd_repo(cmd.clone());
    }
    // self 子命令作用于装在机器上的程序本身，同样不依赖模板目录
    if let Command::SelfCmd { cmd } = &cli.command {
        return cmd_self(cmd.clone());
    }
    // mcp 子命令作用于机器上的客户端配置，不依赖模板目录，提前分发
    if let Command::Mcp { cmd } = &cli.command {
        return cmd_mcp(cmd.clone());
    }
    // gate 子命令作用于当前仓库的门禁检查，同样不依赖模板目录，提前分发
    if let Command::Gate { args } = &cli.command {
        let code =
            pengj_gate::cli::run(args).map_err(|message| anyhow::anyhow!("[gate] {message}"))?;
        if code != 0 {
            update::exit(i32::from(code));
        }
        return Ok(());
    }
    // subrepo 子命令读取项目里的子仓库登记表，同样不依赖模板目录，提前分发
    if let Command::Subrepo { cmd } = &cli.command {
        return cmd_subrepo(cmd.clone());
    }
    // worktree 子命令只作用于项目目录（推导共享编译目录、新建工作树），不依赖模板目录
    if let Command::Worktree { cmd } = &cli.command {
        return cmd_worktree(cmd.clone());
    }
    // doctor 子命令读取项目 manifest，不依赖模板目录，提前分发
    if let Command::Doctor {
        dir,
        fix,
        lang,
        json,
    } = &cli.command
    {
        return cmd_doctor(dir, *fix, lang.as_deref(), *json);
    }
    // setup 子命令读取项目 manifest，不依赖模板目录，提前分发
    if let Command::Setup {
        dir,
        dry_run,
        ephemeral,
        no_ephemeral,
        lang,
        json,
    } = &cli.command
    {
        return cmd_setup(
            dir,
            *dry_run,
            *ephemeral,
            *no_ephemeral,
            lang.as_deref(),
            *json,
        );
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
        Command::AddLayer {
            dir,
            layers,
            sync_skills,
        } => cmd_add_layer(&templates, &dir, &layers, sync_skills),
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
                update::exit(1);
            }
            Ok(())
        }
        Command::CheckLang { dir, lang, json } => {
            let override_lang = match lang.as_deref() {
                None => None,
                Some("zh") => Some(pengj_core::Lang::Zh),
                Some("en") => Some(pengj_core::Lang::En),
                Some(other) => anyhow::bail!("无效语言：{other}（可选 zh/en）"),
            };
            let has_violations = cmd_check_lang(&dir, override_lang, json)?;
            if has_violations {
                update::exit(1);
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
        Command::Repo { .. } => unreachable!("repo 子命令已在加载模板前分发"),
        Command::Mcp { .. } => unreachable!("mcp 子命令已在加载模板前分发"),
        Command::Gate { .. } => unreachable!("gate 子命令已在加载模板前分发"),
        Command::Subrepo { .. } => unreachable!("subrepo 子命令已在加载模板前分发"),
        Command::Worktree { .. } => unreachable!("worktree 子命令已在加载模板前分发"),
        Command::Doctor { .. } => unreachable!("doctor 子命令已在加载模板前分发"),
        Command::Setup { .. } => unreachable!("setup 子命令已在加载模板前分发"),
        Command::CheckUpdate { .. } => unreachable!("check-update 子命令已在加载模板前分发"),
        Command::SelfCmd { .. } => unreachable!("self 子命令已在加载模板前分发"),
        Command::BackgroundUpdateCheck => unreachable!("后台更新检查已在入口处分发"),
    }
}

fn cmd_list_layers(templates: &pengj_core::Templates, json: bool) -> anyhow::Result<()> {
    let layers = templates.list_layers().context("读取层列表失败")?;
    if json {
        println!("{}", serde_json::to_string_pretty(&layers)?);
        return Ok(());
    }
    if layers.is_empty() {
        if json {
            println!("[]");
            return Ok(());
        }
        // 空模板根永远不是「正常的空状态」：指错目录后 `update` 会把整份清单
        // 当上游删除。这里直接报错，给出可执行的下一步。
        anyhow::bail!(
            "模板根 {} 下没有任何层目录（含 layer.toml 的子目录）——--templates 要指向 templates/ 本身，而不是仓库根",
            templates.root().display()
        );
    }
    println!("{:<10} {:<16} {:<40} 文件数", "ID", "名称", "描述");
    for l in layers {
        let deps = if l.depends.is_empty() {
            "-".to_string()
        } else {
            l.depends.join(",")
        };
        let skills = if l.skills.is_empty() {
            "-".to_string()
        } else {
            l.skills.join(",")
        };
        println!(
            "{:<10} {:<16} {:<40} {}  (依赖: {}  配套技能: {})",
            l.id, l.name, l.description, l.file_count, deps, skills
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
    for f in &report.docs_synced {
        println!("  [说明刷新] {f}（只换了模板说明块，你的配置值未被改动）");
    }
    for name in &report.dropped_skills {
        println!("  [清理] 技能 {name}（模板中已无此技能，已从技能清单移除）");
    }
    print_migrations(&report.migrations);
    Ok(())
}

/// 打印跨过的破坏性模板变更。这些变更引擎无法自动迁移，只能由人看一眼。
fn print_migrations(notes: &[pengj_core::MigrationNote]) {
    if notes.is_empty() {
        return;
    }
    println!();
    println!("迁移提示 —— 这次更新跨过了破坏性变更，模板不会替你改，请确认：");
    for note in notes {
        println!("  * v{}：{}", note.version, note.summary);
        println!("    → {}", note.action);
    }
}

fn cmd_add_layer(
    templates: &pengj_core::Templates,
    dir: &Path,
    layers: &[String],
    sync_skills: bool,
) -> anyhow::Result<()> {
    if layers.is_empty() {
        anyhow::bail!("必须指定至少一个层，如 --layers just,bque");
    }
    let opts = pengj_core::UpdateOptions { sync_skills };
    let report = pengj_core::add_layers_to_project(templates, dir, layers, &opts)
        .with_context(|| format!("增量加层 {} 失败", dir.display()))?;

    println!(
        "项目: {}（层: {}）",
        report.project_name,
        report.layers.join(" -> ")
    );
    if !report.added_layers.is_empty() {
        println!("新增层: {}", report.added_layers.join(", "));
    }
    if !report.added_skills.is_empty() {
        println!("联动补齐技能: {}", report.added_skills.join(", "));
    }
    let u = &report.update;
    println!(
        "更新 {} 个文件，新增 {} 个，未变 {} 个",
        u.updated.len(),
        u.created.len(),
        u.unchanged
    );
    for f in &u.updated {
        println!("  [更新] {f}");
    }
    for f in &u.created {
        println!("  [新增] {f}");
    }
    for c in &u.conflicted {
        println!("  [冲突] {} — {}", c.path, c.reason);
    }
    for f in &u.needs_review {
        println!("  [待复核] {f}");
    }
    for f in &u.docs_synced {
        println!("  [说明刷新] {f}（只换了模板说明块，你的配置值未被改动）");
    }
    for name in &u.dropped_skills {
        println!("  [清理] 技能 {name}（模板中已无此技能，已从技能清单移除）");
    }
    print_migrations(&u.migrations);
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
    let mut missing_count = 0;
    let mut orphan_count = 0;
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
                missing_count += 1;
                println!("  [本地文件缺失] {} (可运行 update 补回)", item.path);
            }
            pengj_core::AuditFileStatus::OrphanInManifest => {
                orphan_count += 1;
                println!("  [上游已废弃]   {}", item.path);
            }
        }
    }

    println!("------------------------------------------------------------");
    println!(
        "统计: {} 个一致, {} 个项目合规定制, {} 个待更新, {} 个本地缺失, {} 个上游已废弃, {} 个违规漂移",
        in_sync_count,
        custom_count,
        upstream_newer_count,
        missing_count,
        orphan_count,
        violations.len()
    );
    if !report.warnings.is_empty() {
        println!("------------------------------------------------------------");
        for w in &report.warnings {
            println!("  [配置提示] {w}");
        }
    }

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

/// 校验项目的文档语言是否跟随 skill_lang 语言轴。
///
/// 这是「文案没翻译」这类基础错误的门禁：en 项目不得出现汉字，zh 项目里
/// 面向人的文案不得整份缺失中文。只审计 manifest 记录的模板托管文件。
fn cmd_check_lang(
    dir: &Path,
    override_lang: Option<pengj_core::Lang>,
    json: bool,
) -> anyhow::Result<bool> {
    let report = pengj_core::audit_project_lang(dir, override_lang)
        .context("校验项目文档语言失败（请先确认已生成项目并有 .pengj-templates/manifest.json）")?;

    if json {
        let violations = pengj_core::format_findings(&report);
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "lang": report.lang.as_str(),
                "filesChecked": report.files_checked,
                "violations": violations,
            }))?
        );
        return Ok(report.has_violations());
    }

    println!(
        "项目 {} 文档语言校验（{}）：检查 {} 个模板托管文件",
        dir.display(),
        report.lang.as_str(),
        report.files_checked
    );
    if !report.has_violations() {
        println!("通过：文案与语言轴一致。");
        return Ok(false);
    }
    eprintln!(
        "发现 {} 处文案语言违规（en 项目不得含汉字；zh 项目文案必须含中文）：",
        report.findings.len()
    );
    for line in pengj_core::format_findings(&report) {
        eprintln!("{line}");
    }
    eprintln!("提示：到模板源补上 {{% if options[\"skill_lang\"] == \"en\" %}}…{{% else %}}…{{% endif %}} 双语分支，再对项目执行 update。");
    Ok(true)
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
        anyhow::bail!("必须指定至少一个层，如 --layers common,gate,agent");
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

fn cmd_self(cmd: SelfCommand) -> anyhow::Result<()> {
    match cmd {
        SelfCommand::Update {
            from_source,
            version,
            force,
            dry_run,
            no_path_refresh,
            workspace,
            timeout,
        } => self_update::run(&self_update::Options {
            from_source,
            version,
            force,
            dry_run,
            no_path_refresh,
            workspace,
            timeout,
        }),
    }
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
            update::exit(1);
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
        update::exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// subrepo：子仓库登记表
// ---------------------------------------------------------------------------

fn cmd_subrepo(cmd: SubrepoCommand) -> anyhow::Result<()> {
    match cmd {
        SubrepoCommand::List { dir, json } => cmd_subrepo_list(&dir, json),
        SubrepoCommand::Check { dir, json } => cmd_subrepo_check(&dir, json),
    }
}

/// 把登记表渲染成人类可读的清单。
///
/// 登记表为空、或条目缺少必填字段时**报错退出**，不输出半成品：
/// 一份「看起来完整」但实际漏了条目的清单，比明确报错更危险。
fn cmd_subrepo_list(dir: &Path, json: bool) -> anyhow::Result<()> {
    let registry = pengj_core::SubrepoRegistry::load(dir)
        .context("读取子仓库登记表失败（登记表不存在，或内容不是合法的 TOML）")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "未找到子仓库登记表: {}（该项目可能未勾选「子仓库」层）",
                pengj_core::SUBREPO_REGISTRY_REL
            )
        })?;

    if registry.entries.is_empty() {
        anyhow::bail!(
            "子仓库登记表为空: {}（请先登记 [[subrepo]] 条目，再渲染清单）",
            pengj_core::SUBREPO_REGISTRY_REL
        );
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&registry)?);
        return Ok(());
    }

    println!(
        "子仓库清单（{} 条，来源: {}）",
        registry.entries.len(),
        pengj_core::SUBREPO_REGISTRY_REL
    );
    println!("------------------------------------------------------------");
    for entry in &registry.entries {
        println!("名称: {}", entry.name);
        println!("  相对路径: {}", entry.path);
        println!(
            "  依赖模式: {}（{}）",
            entry.mode.as_str(),
            entry.mode.describe()
        );
        match entry.mode {
            pengj_core::DependencyMode::Submodule => {
                println!(
                    "  上游跟踪分支: {}",
                    entry.upstream.as_deref().unwrap_or("-")
                );
            }
            pengj_core::DependencyMode::Local => {
                println!(
                    "  本地同级目录: {}",
                    entry.local_dir.as_deref().unwrap_or("-")
                );
            }
            pengj_core::DependencyMode::Fetch => {
                println!("  上游跟踪分支: -（按版本拉取，版本锁定在宿主构建系统里）");
            }
        }
        if entry.has_workspace {
            println!("  自带工作区定义: 是（宿主工作区必须排除它）");
        }
    }
    Ok(())
}

/// 子仓库体检：逐条结论 + 分级退出码。
///
/// 检查逻辑在 `pengj_core::subrepo_check` 里（单一实现），自检脚本只负责转发到这里——
/// 与提交门禁「只有一套实现」的既有原则一致。
fn cmd_subrepo_check(dir: &Path, json: bool) -> anyhow::Result<()> {
    let report = pengj_core::check_project(dir)
        .context("子仓库体检失败（登记表不存在，或内容不是合法的登记表）")?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("子仓库体检（{}）", report.root);
        println!("------------------------------------------------------------");
        for item in &report.items {
            let label = match item.level {
                pengj_core::CheckLevel::Pass => "通过",
                pengj_core::CheckLevel::Warn => "警告",
                pengj_core::CheckLevel::Fail => "失败",
            };
            println!("[{}] {}：{}", label, item.name, item.detail);
            if let Some(remedy) = &item.remedy {
                println!("        → {remedy}");
            }
        }
        let failures = report
            .items
            .iter()
            .filter(|i| i.level == pengj_core::CheckLevel::Fail)
            .count();
        let warnings = report
            .items
            .iter()
            .filter(|i| i.level == pengj_core::CheckLevel::Warn)
            .count();
        println!("------------------------------------------------------------");
        println!(
            "共 {} 项检查：{} 个失败，{} 个警告",
            report.items.len(),
            failures,
            warnings
        );
    }

    let code = report.exit_code();
    if code != 0 {
        update::exit(code);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// worktree：共享编译目录与新建工作树
// ---------------------------------------------------------------------------

/// 展示用路径：剥掉 Windows 的 `\\?\` 逐字前缀（`canonicalize` 会加上它，
/// 打印给人看只会变成噪音）。
fn display_path(path: &Path) -> String {
    let text = path.display().to_string();
    #[cfg(windows)]
    {
        text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
    }
    #[cfg(not(windows))]
    {
        text
    }
}

fn cmd_worktree(cmd: WorktreeCommand) -> anyhow::Result<()> {
    match cmd {
        WorktreeCommand::TargetDir { dir, ensure, json } => {
            cmd_worktree_target_dir(&dir, ensure, json)
        }
        WorktreeCommand::Check { dir, fix, json } => cmd_worktree_check(&dir, fix, json),
        WorktreeCommand::Create { name, dir, json } => cmd_worktree_create(&dir, &name, json),
    }
}

/// 打印共享编译目录的绝对路径。
///
/// 非 JSON 模式下**只打印路径**：构建配方把它的标准输出直接当作 `CARGO_TARGET_DIR`
/// 的值，多打一行说明就会毒化环境变量。
fn cmd_worktree_target_dir(dir: &Path, ensure: bool, json: bool) -> anyhow::Result<()> {
    let root = repo_root(dir)?;
    let (target, state) = if ensure {
        pengj_core::ensure_shared_target(&root)?
    } else {
        pengj_core::check_shared_target(&root)
    };
    if json {
        println!(
            "{}",
            serde_json::json!({
                "path": target.path.to_string_lossy(),
                "main_worktree": display_path(&target.main_worktree),
                "state": state.as_str(),
            })
        );
        return Ok(());
    }
    println!("{}", target.path.display());
    Ok(())
}

/// 校验共享目录：存在、可写，且没有指向别处的覆盖值。
fn cmd_worktree_check(dir: &Path, fix: bool, json: bool) -> anyhow::Result<()> {
    let root = repo_root(dir)?;
    let (target, state) = if fix {
        pengj_core::ensure_shared_target(&root)?
    } else {
        pengj_core::check_shared_target(&root)
    };
    let override_path = pengj_core::conflicting_override(&target);

    if json {
        println!(
            "{}",
            serde_json::json!({
                "root": display_path(&root),
                "path": target.path.to_string_lossy(),
                "main_worktree": display_path(&target.main_worktree),
                "state": state.as_str(),
                "override_path": override_path,
            })
        );
    } else {
        println!("共享编译目录（{}）", display_path(&root));
        println!("------------------------------------------------------------");
        println!("  路径: {}", target.path.display());
        println!("  主工作树: {}", display_path(&target.main_worktree));
        println!("  状态: {}", state.describe());
        println!(
            "  说明: 所有工作树共用这一份产物；在主工作树里 cargo clean 会一并清掉\
             各工作树的中间产物（需全量重建）"
        );
        if let Some(path) = &override_path {
            println!(
                "  警告: 环境变量 CARGO_TARGET_DIR 指向 {path}，与本机推导值不一致；\
                 裸跑 cargo 会落到别处，请经由 just 配方构建或清掉这个覆盖值"
            );
        }
        if state == pengj_core::SharedTargetState::Missing {
            println!("  → 修复：pengj-templates-cli worktree check --dir . --fix");
        }
    }

    if state != pengj_core::SharedTargetState::Ready {
        update::exit(1);
    }
    // 目录就绪但环境里有个指向别处的覆盖值：不阻断（配方会把值盖回去），
    // 但必须让人看见——否则「裸跑 cargo」的工作树会悄悄落到另一个目录。
    if override_path.is_some() {
        update::exit(2);
    }
    Ok(())
}

/// 新建工作树：分支 + 目录一次做完，接线共享编译目录并补齐子模块。
fn cmd_worktree_create(dir: &Path, name: &str, json: bool) -> anyhow::Result<()> {
    let root = repo_root(dir)?;
    let report = pengj_core::create_worktree(&root, name)?;

    if json {
        let submodules: Vec<serde_json::Value> = report
            .submodules
            .iter()
            .map(|(name, ready)| serde_json::json!({ "name": name, "ready": ready }))
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "path": report.path.to_string_lossy(),
                "branch": report.branch,
                "shared_target": report.shared_target.to_string_lossy(),
                "submodules": submodules,
            })
        );
        return Ok(());
    }

    println!("工作树已创建");
    println!("------------------------------------------------------------");
    println!("  路径: {}", report.path.display());
    println!("  分支: {}", report.branch);
    println!("  共享编译目录: {}", report.shared_target.display());
    if report.submodules.is_empty() {
        println!("  子模块: 无");
    } else {
        for (name, ready) in &report.submodules {
            println!(
                "  子模块 {name}: {}",
                if *ready { "就绪" } else { "未就绪" }
            );
        }
        if report.submodules.iter().any(|(_, ready)| !ready) {
            println!("  → 补齐：pengj-templates-cli repo init --dir .");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// repo：仓库级初始化
// ---------------------------------------------------------------------------

fn cmd_repo(cmd: RepoCommand) -> anyhow::Result<()> {
    match cmd {
        RepoCommand::Status { dir, links, json } => {
            let root = repo_root(&dir)?;
            let specs = build_links(&root, &links)?;
            let report = pengj_core::repo_status(&root, &specs)?;
            print_repo_report(&report, json);
            if !report.ok {
                update::exit(1);
            }
            Ok(())
        }
        RepoCommand::Init {
            dir,
            links,
            no_submodules,
            json,
        } => {
            let root = repo_root(&dir)?;
            let specs = build_links(&root, &links)?;
            let report = pengj_core::repo_init(&root, &specs, !no_submodules)?;
            print_repo_report(&report, json);
            if !report.ok {
                update::exit(1);
            }
            Ok(())
        }
    }
}

fn repo_root(dir: &Path) -> anyhow::Result<PathBuf> {
    dir.canonicalize()
        .with_context(|| format!("目录不存在: {}", dir.display()))
}

fn build_links(root: &Path, raw: &[String]) -> anyhow::Result<Vec<pengj_core::LinkSpec>> {
    let mut links = pengj_core::default_links(root);
    for value in raw {
        let spec =
            pengj_core::LinkSpec::parse(value).map_err(|error| anyhow::anyhow!("{error}"))?;
        match links.iter_mut().find(|existing| existing.link == spec.link) {
            Some(existing) => *existing = spec,
            None => links.push(spec),
        }
    }
    Ok(links)
}

fn print_repo_report(report: &pengj_core::RepoReport, json: bool) {
    if json {
        match serde_json::to_string_pretty(report) {
            Ok(text) => println!("{text}"),
            Err(error) => eprintln!("[repo] JSON 序列化失败: {error}"),
        }
        return;
    }
    for item in &report.items {
        let label = match item.status {
            pengj_core::RepoStatus::Ok => "ok",
            pengj_core::RepoStatus::Missing => "missing",
            pengj_core::RepoStatus::Created => "created",
            pengj_core::RepoStatus::Conflict => "conflict",
            pengj_core::RepoStatus::Skipped => "skipped",
        };
        println!("[repo] {label:<8} {}: {}", item.name, item.detail);
    }
    println!("[repo] {}", if report.ok { "就绪" } else { "存在问题" });
}

// ---------------------------------------------------------------------------
// doctor：项目环境自检
// ---------------------------------------------------------------------------

fn cmd_doctor(dir: &Path, fix: bool, lang: Option<&str>, json: bool) -> anyhow::Result<()> {
    let override_lang = match lang {
        None => None,
        Some("zh") => Some(pengj_core::Lang::Zh),
        Some("en") => Some(pengj_core::Lang::En),
        Some(other) => anyhow::bail!("无效语言：{other}（可选 zh/en）"),
    };

    let opts = pengj_core::DoctorOptions {
        fix,
        lang: override_lang,
    };

    let report = match pengj_core::run_doctor(dir, &opts) {
        Ok(report) => report,
        Err(pengj_core::CoreError::MissingManifest(_)) => {
            anyhow::bail!(
                "未找到 .pengj-templates/manifest.json（{} 尚未由 pengj-templates 纳管）；\
                 请先运行 `pengj-templates-cli adopt --dir {} --layers <层清单>`",
                dir.display(),
                dir.display()
            );
        }
        Err(err) => return Err(err.into()),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let effective_lang = pengj_core::Lang::from_skill_lang(Some(report.lang.as_str()));
        print!(
            "{}",
            pengj_core::format_doctor_report(&report, effective_lang)
        );
    }

    let code = report.exit_code();
    if code != 0 {
        update::exit(code);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// setup：跨平台一键装机
// ---------------------------------------------------------------------------

fn cmd_setup(
    dir: &Path,
    dry_run: bool,
    ephemeral: bool,
    no_ephemeral: bool,
    lang: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let override_lang = match lang {
        None => None,
        Some("zh") => Some(pengj_core::Lang::Zh),
        Some("en") => Some(pengj_core::Lang::En),
        Some(other) => anyhow::bail!("无效语言：{other}（可选 zh/en）"),
    };

    let ephemeral_override = if ephemeral {
        Some(true)
    } else if no_ephemeral {
        Some(false)
    } else {
        None
    };

    let opts = pengj_core::SetupOptions {
        dry_run,
        ephemeral: ephemeral_override,
        lang: override_lang,
    };

    let report = match pengj_core::run_setup(dir, &opts) {
        Ok(report) => report,
        Err(pengj_core::CoreError::MissingManifest(_)) => {
            anyhow::bail!(
                "未找到 .pengj-templates/manifest.json（{} 尚未由 pengj-templates 纳管）；\
                 请先运行 `pengj-templates-cli adopt --dir {} --layers <层清单>`",
                dir.display(),
                dir.display()
            );
        }
        Err(err) => return Err(err.into()),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let effective_lang = pengj_core::Lang::from_skill_lang(Some(report.lang.as_str()));
        print!(
            "{}",
            pengj_core::format_setup_report(&report, effective_lang)
        );
    }

    let code = report.exit_code();
    if code != 0 {
        update::exit(code);
    }
    Ok(())
}

#[cfg(test)]
mod setup_cli_tests {
    use super::*;

    #[test]
    fn setup子命令参数解析支持dry_run与临时环境覆盖() {
        let cli = Cli::try_parse_from([
            "pengj-templates-cli",
            "setup",
            "--dir",
            "/tmp/demo",
            "--dry-run",
            "--ci",
            "--lang",
            "en",
            "--json",
        ])
        .expect("--ci 别名与 --dry-run/--lang/--json 应可解析");

        match cli.command {
            Command::Setup {
                dir,
                dry_run,
                ephemeral,
                no_ephemeral,
                lang,
                json,
            } => {
                assert_eq!(dir, PathBuf::from("/tmp/demo"));
                assert!(dry_run);
                assert!(ephemeral);
                assert!(!no_ephemeral);
                assert_eq!(lang.as_deref(), Some("en"));
                assert!(json);
            }
            _ => panic!("预期解析为 Command::Setup"),
        }

        let cli_no_eph = Cli::try_parse_from(["pengj-templates-cli", "setup", "--no-ephemeral"])
            .expect("--no-ephemeral 应可解析");
        match cli_no_eph.command {
            Command::Setup {
                ephemeral,
                no_ephemeral,
                ..
            } => {
                assert!(!ephemeral);
                assert!(no_ephemeral);
            }
            _ => panic!("预期解析为 Command::Setup"),
        }

        let conflict = Cli::try_parse_from([
            "pengj-templates-cli",
            "setup",
            "--ephemeral",
            "--no-ephemeral",
        ]);
        assert!(
            conflict.is_err(),
            "--ephemeral 与 --no-ephemeral 同时传入应报错"
        );
    }
}
