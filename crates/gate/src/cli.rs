//! The `gate` command-line front end.
//!
//! The checks themselves live in the library so the integration tests can
//! call them directly. This module holds only the argument handling, which
//! the template CLI exposes as `pengj-templates-cli gate ...` — the single
//! public entry point of the gate.

use crate::config::Mode;
use crate::{commit, config, fmt, git, hooks, lang_guard, prepush};

/// The help text, shown by `pengj-templates-cli gate --help`.
pub const HELP: &str = "\
pengj-templates-cli gate - commit gate for Rust repositories

USAGE
  pengj-templates-cli gate [audit] [FLAGS] [FILES...]   code-language guard (default: whole repository)
  pengj-templates-cli gate fmt [FILES...]               rustfmt the given files and re-stage them
  pengj-templates-cli gate commit-msg FILE              validate a commit message
  pengj-templates-cli gate hook pre-commit              staged rustfmt + language guard
  pengj-templates-cli gate hook pre-push                fmt --check + language guard + prepush.commands
  pengj-templates-cli gate hook commit-msg FILE         commit message validation
  pengj-templates-cli gate hooks install                point core.hooksPath at .githooks
  pengj-templates-cli gate check-commits [--base REF]   validate the commit messages in a range
  pengj-templates-cli gate selftest                     run the built-in rule self-test

FLAGS
  --mode=english|chinese   override `.pengj-templates/gate.toml` (env: PENGJ_CODE_LANG)
  --max-ratio=N            string violation tolerance per file (default 0.01)
  --max-ident-ratio=N      identifier violation tolerance per file (default 0)
  --json                   machine-readable report
  --list                   print every violation, not just the first few
";

struct Settings {
    mode: Mode,
    mode_source: String,
    max_ratio: f64,
    max_ident_ratio: f64,
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn first_number(config: &config::Config, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| config.number(key))
}

fn checked_ratio(name: &str, value: f64) -> Result<f64, String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(format!(
            "invalid {name}={value} (expected a number in [0, 1])"
        ));
    }
    Ok(value)
}

fn resolve(
    mode_flag: Option<&str>,
    ratio_flag: Option<f64>,
    ident_flag: Option<f64>,
) -> Result<Settings, String> {
    let cwd = std::env::current_dir().unwrap_or_default();
    let (config, path) = config::load(&cwd)?;

    let (mode, mode_source) = if let Some(flag) = mode_flag {
        let mode = Mode::parse(flag)
            .ok_or_else(|| format!("invalid --mode={flag} (expected english|chinese)"))?;
        (mode, "--mode".to_string())
    } else if let Some(raw) = env_value("PENGJ_CODE_LANG") {
        let mode = Mode::parse(&raw)
            .ok_or_else(|| format!("invalid PENGJ_CODE_LANG={raw} (expected english|chinese)"))?;
        (mode, "PENGJ_CODE_LANG".to_string())
    } else if let Some(mode) = config.mode() {
        let source = path
            .as_ref()
            .map(|path| format!("{} mode", path.display()))
            .unwrap_or_else(|| "mode".to_string());
        (mode, source)
    } else {
        return Err(format!(
            "cannot determine the code language: pass --mode=english|chinese, set PENGJ_CODE_LANG, or add `mode = \"english\"` to {}",
            config::CONFIG_FILE
        ));
    };

    let max_ratio = match ratio_flag {
        Some(value) => value,
        None => match env_value("PENGJ_LANG_MAX_RATIO") {
            Some(raw) => raw.parse().map_err(|_| {
                format!("invalid PENGJ_LANG_MAX_RATIO={raw} (expected a number in [0, 1])")
            })?,
            None => first_number(&config, &["lang.max_ratio", "max_ratio"]).unwrap_or(0.01),
        },
    };
    let max_ident_ratio = match ident_flag {
        Some(value) => value,
        None => match env_value("PENGJ_LANG_MAX_IDENT_RATIO") {
            Some(raw) => raw.parse().map_err(|_| {
                format!("invalid PENGJ_LANG_MAX_IDENT_RATIO={raw} (expected a number in [0, 1])")
            })?,
            None => {
                first_number(&config, &["lang.max_ident_ratio", "max_ident_ratio"]).unwrap_or(0.0)
            }
        },
    };

    Ok(Settings {
        mode,
        mode_source,
        max_ratio: checked_ratio("--max-ratio", max_ratio)?,
        max_ident_ratio: checked_ratio("--max-ident-ratio", max_ident_ratio)?,
    })
}

struct AuditArgs {
    files: Vec<String>,
    mode: Option<String>,
    max_ratio: Option<f64>,
    max_ident_ratio: Option<f64>,
    json: bool,
    list: bool,
}

fn parse_audit_args(args: &[String]) -> Result<AuditArgs, String> {
    let mut parsed = AuditArgs {
        files: Vec::new(),
        mode: None,
        max_ratio: None,
        max_ident_ratio: None,
        json: false,
        list: false,
    };
    for arg in args {
        if let Some(value) = arg.strip_prefix("--mode=") {
            parsed.mode = Some(value.to_string());
        } else if let Some(value) = arg.strip_prefix("--max-ratio=") {
            parsed.max_ratio = Some(value.parse().map_err(|_| {
                format!("invalid --max-ratio={value} (expected a number in [0, 1])")
            })?);
        } else if let Some(value) = arg.strip_prefix("--max-ident-ratio=") {
            parsed.max_ident_ratio = Some(value.parse().map_err(|_| {
                format!("invalid --max-ident-ratio={value} (expected a number in [0, 1])")
            })?);
        } else if arg == "--json" {
            parsed.json = true;
        } else if arg == "--list" {
            parsed.list = true;
        } else if arg.starts_with("--") {
            return Err(format!("unknown flag: {arg}"));
        } else {
            parsed.files.push(arg.clone());
        }
    }
    Ok(parsed)
}

fn options_for(
    files: Vec<String>,
    settings: &Settings,
    json: bool,
    list: bool,
) -> lang_guard::AuditOptions {
    lang_guard::AuditOptions {
        files,
        mode: settings.mode,
        mode_source: settings.mode_source.clone(),
        max_ratio: settings.max_ratio,
        max_ident_ratio: settings.max_ident_ratio,
        json,
        list,
    }
}

fn audit_command(args: &[String]) -> Result<u8, String> {
    let parsed = parse_audit_args(args)?;
    let settings = resolve(
        parsed.mode.as_deref(),
        parsed.max_ratio,
        parsed.max_ident_ratio,
    )?;
    let options = options_for(parsed.files, &settings, parsed.json, parsed.list);
    Ok(lang_guard::audit(&options) as u8)
}

fn hook_pre_commit() -> Result<u8, String> {
    let files = git::staged_rust_files()?;
    if files.is_empty() {
        return Ok(0);
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    let (config, _) = config::load(&cwd)?;

    if !config.is_disabled("fmt") {
        fmt::run(&files)?;
    }
    if config.is_disabled("lang-guard") {
        return Ok(0);
    }

    let settings = resolve(None, None, None)?;
    let options = options_for(files, &settings, false, false);
    Ok(lang_guard::audit(&options) as u8)
}

/// 推送前的最后一道本地闸门。
///
/// 为什么不是在 `pre-commit` 里查：`git am`、`git cherry-pick`、`git rebase` 都不触发
/// `pre-commit`，批量落补丁这条路径从来跑不到它。详见 `prepush` 模块头部。
fn hook_pre_push() -> Result<u8, String> {
    // git 把待推的引用写在 stdin 上，先原样报出来——一次推多个分支时，失败信息得能
    // 对上是哪一个。
    let refs = prepush::read_refs();
    prepush::report_refs(&refs);

    let cwd = std::env::current_dir().unwrap_or_default();

    // 若本次推送范围内全部为纯文档文件（零代码/构建文件变更），直接放行，避免纯文档推送
    // 触发无关的 `cargo fmt --check`、代码语言守卫或重型自定义命令。
    if !refs.is_empty() {
        if let Some(files) = prepush::changed_files_for_refs(&cwd, &refs) {
            if !prepush::has_code_changes(&files) {
                println!(
                    "[gate] pre-push: docs-only changes ({} file(s)), skipping fmt, lang-guard and custom commands",
                    files.len()
                );
                return Ok(0);
            }
        }
    }
    let (config, _) = config::load(&cwd)?;

    // 内置两项与 `gate.toml` 的开关一一对应，关掉的方式和 pre-commit 一致。
    if !config.is_disabled("fmt") {
        prepush::check_fmt()?;
    }
    if !config.is_disabled("lang-guard") {
        let settings = resolve(None, None, None)?;
        // 空文件列表 = 整个工作树；推送覆盖整个仓，不是几个暂存文件。
        let options = options_for(Vec::new(), &settings, false, false);
        let code = lang_guard::audit(&options);
        if code != 0 {
            return Ok(code as u8);
        }
    }

    let commands = config
        .list("prepush.commands")
        .map(<[String]>::to_vec)
        .unwrap_or_default();
    if !commands.is_empty() && prepush::skip_commands_requested() {
        println!(
            "[gate] pre-push: skipping {} custom command(s) ({} is set)",
            commands.len(),
            prepush::SKIP_COMMANDS_ENV
        );
    } else {
        let code = prepush::run_commands(&commands)?;
        if code != 0 {
            return Ok(code);
        }
    }

    println!("[gate] pre-push passed");
    Ok(0)
}

fn self_test() -> u8 {
    let cases = lang_guard::self_test_cases();
    let mut failed = 0;
    for (name, ok) in &cases {
        println!("{} {name}", if *ok { "ok  " } else { "FAIL" });
        if !ok {
            failed += 1;
        }
    }
    if failed > 0 {
        eprintln!("[gate] self-test failed: {failed}/{}", cases.len());
        1
    } else {
        println!("[gate] self-test passed: {} cases", cases.len());
        0
    }
}

/// Validate every commit message in a range.
///
/// This is the server-side counterpart of the `commit-msg` hook: it needs no
/// local setup at all, so a pull request is guarded even when the author
/// never installed the hooks.
fn check_commits(rest: &[String]) -> Result<u8, String> {
    let mut base: Option<String> = None;
    let mut head: Option<String> = None;
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        let value = rest.get(index + 1).ok_or_else(|| {
            format!("{flag} needs a value; expected a git ref, e.g. --base origin/main")
        })?;
        match flag {
            "--base" => base = Some(value.clone()),
            "--head" => head = Some(value.clone()),
            other => {
                return Err(format!(
                    "unknown flag for check-commits: {other}; expected --base or --head"
                ))
            }
        }
        index += 2;
    }

    let report = commit::check_commits(base.as_deref(), head.as_deref())?;

    if report.range.is_empty() {
        println!(
            "[gate] the commit gate is disabled in .pengj-templates/gate.toml; nothing to check"
        );
        return Ok(0);
    }
    if report.checked == 0 {
        println!("[gate] no commits in {}; nothing to check", report.range);
        return Ok(0);
    }

    for rejected in &report.rejected {
        let short = &rejected.sha[..rejected.sha.len().min(8)];
        eprintln!("[gate] commit {short} was rejected: {}", rejected.subject);
        for issue in &rejected.issues {
            eprintln!("  - {issue}");
        }
    }

    if !report.rejected.is_empty() {
        eprintln!(
            "[gate] {}/{} commit(s) in {} were rejected",
            report.rejected.len(),
            report.checked,
            report.range
        );
        Ok(1)
    } else {
        println!(
            "[gate] commit messages passed: {} commit(s) checked in {}",
            report.checked, report.range
        );
        Ok(0)
    }
}

/// Run the gate with the given arguments and return the process exit code.
///
/// `pengj-templates-cli gate ...` is the only public entry point, so there is
/// exactly one parser to keep honest.
pub fn run(args: &[String]) -> Result<u8, String> {
    let Some(first) = args.first() else {
        return audit_command(&[]);
    };
    let rest = &args[1..];

    match first.as_str() {
        "-h" | "--help" | "help" => {
            println!("{HELP}");
            Ok(0)
        }
        "audit" | "check" => audit_command(rest),
        "selftest" | "--self-test" => Ok(self_test()),
        "check-commits" => check_commits(rest),
        "fmt" => {
            let files = if rest.is_empty() {
                git::staged_rust_files()?
            } else {
                rest.to_vec()
            };
            fmt::run(&files)?;
            Ok(0)
        }
        "commit-msg" => {
            let path = rest
                .first()
                .ok_or_else(|| "commit-msg needs a message file".to_string())?;
            commit::validate_file(path)
        }
        "hook" => match rest.first().map(String::as_str) {
            Some("pre-commit") => hook_pre_commit(),
            Some("pre-push") => hook_pre_push(),
            Some("commit-msg") => {
                let path = rest
                    .get(1)
                    .ok_or_else(|| "hook commit-msg needs a message file".to_string())?;
                commit::validate_file(path)
            }
            _ => Err("unknown hook; expected pre-commit, pre-push or commit-msg".to_string()),
        },
        "hooks" => match rest.first().map(String::as_str) {
            Some("install") => {
                if let Some(extra) = rest.get(1) {
                    return Err(format!("unknown hooks argument: {extra}"));
                }
                hooks::install()?;
                Ok(0)
            }
            _ => Err("unknown hooks command; expected `hooks install`".to_string()),
        },
        other if other.starts_with('-') => audit_command(args),
        // Bare arguments keep working as file paths, the way the old
        // `check-lang.mjs <files...>` invocation did.
        _ => audit_command(args),
    }
}
