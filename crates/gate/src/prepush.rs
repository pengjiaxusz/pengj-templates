//! pre-push：推送前的最后一道本地闸门。
//!
//! 这个钩子的存在理由只有一个——`pre-commit` 覆盖不到批量落补丁。实测的钩子触发面：
//!
//! | 命令 | `pre-commit` | `commit-msg` | `pre-applypatch` |
//! | :--- | :--- | :--- | :--- |
//! | `git commit` / `--amend` | 触发 | 触发 | — |
//! | `git am` | 不触发 | 不触发 | 触发 |
//! | `git cherry-pick` | 不触发 | 不触发 | — |
//! | `git rebase` | 不触发 | 不触发 | — |
//!
//! 也就是说「提交时检查」只对敲命令的人有效，而批量落补丁这类最需要检查的路径反而
//! 裸奔，问题一路藏到 CI 才暴露。`pre-push` 是唯一覆盖所有造历史方式的钩子：不管
//! 提交是敲出来的、am 进来的还是 rebase 出来的，要发出去都得先过这里。
//!
//! 内置检查只有两项，都是秒级且「CI 会跑、本地最容易漏」的：`cargo fmt --check`（见
//! [`check_fmt`]）与代码语言守卫（在 `cli` 里编排，因为它要读 `gate.toml` 的模式）。
//! `prepush.commands` 默认留空，重型检查（clippy、全量测试等）应交由 CI；若项目显式
//! 声明了额外命令，受限环境可通过 `PENGJ_GATE_SKIP_PREPUSH_COMMANDS=1` 跳过它们，
//! 同时保留内置秒级门禁。

use std::io::{IsTerminal, Read};
use std::path::Path;
use std::process::Command;

/// `git push` 从 stdin 递过来的一行：本地引用、本地 sha、远端引用、远端 sha。
pub struct PushRef {
    pub local_ref: String,
    pub local_sha: String,
    pub remote_ref: String,
    pub remote_sha: String,
}

/// 解析 `git push` 的 stdin。
///
/// 每行四个字段，tab 分隔（也可能是空格，统一按空白切）。删除分支的行本地 sha 全零，
/// 没有内容要检查，直接丢掉；字段不足四个的行同样丢掉——宁可少报一行，也不要因为
/// 一个陌生格式把推送拦死。
pub fn parse_refs(stdin: &str) -> Vec<PushRef> {
    let mut refs = Vec::new();
    for line in stdin.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        let local_sha = fields[1];
        if local_sha.chars().all(|c| c == '0') {
            continue;
        }
        refs.push(PushRef {
            local_ref: fields[0].to_string(),
            local_sha: local_sha.to_string(),
            remote_ref: fields[2].to_string(),
            remote_sha: fields[3].to_string(),
        });
    }
    refs
}

/// 读 stdin 里的引用列表。
///
/// 钩子被 git 调用时 stdin 是管道；人手动执行时 stdin 是终端，此时读会一直等下去，
/// 所以先判终端再决定读不读。
pub fn read_refs() -> Vec<PushRef> {
    if std::io::stdin().is_terminal() {
        return Vec::new();
    }
    let mut buffer = String::new();
    if std::io::stdin().read_to_string(&mut buffer).is_err() {
        return Vec::new();
    }
    parse_refs(&buffer)
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(8)]
}

/// 把将要推送的引用打出来。
///
/// 这一段不是装饰：push 可能一次推多个分支，失败时必须能对上是哪一个。
pub fn report_refs(refs: &[PushRef]) {
    for item in refs {
        let remote = if item.remote_sha.chars().all(|c| c == '0') {
            "new".to_string()
        } else {
            short(&item.remote_sha).to_string()
        };
        println!(
            "[gate] pre-push: {} -> {} ({} .. {})",
            item.local_ref,
            item.remote_ref,
            remote,
            short(&item.local_sha)
        );
    }
}

/// 判断路径是否属于纯文档类文件（与模板 `gate.yml` / `detect-changes.yml` 保持同一口径）。
///
/// 仅匹配根目录 `*.md`、`docs/`、`.agents/`、`.github/pull_request_template.md` 以及根目录 `LICENSE*`；
/// 子目录下的 `.md`（如嵌入二进制的 `templates/**/*.md`）不视为纯文档，避免误跳过构建检查。
pub fn is_doc_only_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let p = normalized.trim().trim_start_matches("./");
    if p.is_empty() {
        return true;
    }
    if p.starts_with("docs/")
        || p.starts_with(".agents/")
        || p == ".github/pull_request_template.md"
    {
        return true;
    }
    if !p.contains('/') && (p.ends_with(".md") || p.starts_with("LICENSE")) {
        return true;
    }
    false
}

/// 判断给定变更文件清单中是否包含非文档代码/构建文件。
pub fn has_code_changes(files: &[String]) -> bool {
    files.iter().any(|f| !is_doc_only_path(f))
}

/// 收集本次 `git push` 引用范围内的全部变更文件路径。
///
/// 任何无法安全确定范围的情况（无引用传入、强推后远端 sha 在本地不存在、git 命令失败）
/// 一律返回 `None`，调用方据此保守执行全量检查，绝不误放行。
pub fn changed_files_for_refs(dir: &Path, refs: &[PushRef]) -> Option<Vec<String>> {
    if refs.is_empty() {
        return None;
    }
    let mut files = Vec::new();
    for item in refs {
        let is_new_branch = item.remote_sha.chars().all(|c| c == '0');
        let output = if is_new_branch {
            // 新分支：统计本地分支相对所有已知远端引用的新增提交所触及的文件。
            Command::new("git")
                .current_dir(dir)
                .args([
                    "log",
                    "--format=",
                    "--name-only",
                    &item.local_sha,
                    "--not",
                    "--remotes",
                ])
                .output()
                .ok()?
        } else {
            let range = format!("{}..{}", item.remote_sha, item.local_sha);
            Command::new("git")
                .current_dir(dir)
                .args(["diff", "--name-only", &range])
                .output()
                .ok()?
        };
        if !output.status.success() {
            return None;
        }
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                files.push(trimmed.to_string());
            }
        }
    }
    Some(files)
}

/// 用 rustfmt 做一次只读检查，与 CI 的 `cargo fmt --check` 完全同一条命令。
///
/// 只读是刻意的：pre-push 阶段改文件已经进不了提交了（`git am` 会把索引重置回它自己
/// 写的那棵树），就地格式化只会留一棵脏工作树。要改就得由人来改、人再来一次。
pub fn check_fmt() -> Result<(), String> {
    // 没有 Cargo.toml 说明不是 Rust 项目，没有格式可查；静默跳过，别拿一条无关错误
    // 去拦一次合法推送。
    if !Path::new("Cargo.toml").exists() {
        return Ok(());
    }
    println!("[gate] pre-push: cargo fmt --check");
    let output = Command::new("cargo")
        .args(["fmt", "--check"])
        .output()
        .map_err(|error| {
            format!("cannot run `cargo fmt`: {error}\n  install the toolchain with rustup")
        })?;
    if output.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let detail = if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    };
    Err(format!(
        "formatting is not clean, and CI's `cargo fmt --check` would fail the same way.\n  \
         run `cargo fmt` (or `just fmt`), review the diff, then push again.\n{detail}"
    ))
}

/// 受限环境下跳过 `prepush.commands`（但保留内置秒级门禁）的环境变量名。
pub const SKIP_COMMANDS_ENV: &str = "PENGJ_GATE_SKIP_PREPUSH_COMMANDS";

/// 解析布尔风格的跳过标记：`1`、`true`、`yes`（忽略大小写与首尾空白）视为开启。
pub fn parse_skip_flag(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

/// 判断当前进程是否请求跳过 `prepush.commands`。
pub fn skip_commands_requested() -> bool {
    std::env::var(SKIP_COMMANDS_ENV)
        .map(|val| parse_skip_flag(&val))
        .unwrap_or(false)
}

/// 按 `prepush.commands` 的顺序跑项目自己的检查。
///
/// 每条写成「程序 + 参数」的普通字符串，按空白切分，不支持引号与管道——钩子里没有
/// 第二套 shell 语义，需要复杂编排就把它包进项目的 just 配方，这里只留一个入口名。
pub fn run_commands(commands: &[String]) -> Result<u8, String> {
    for command in commands {
        let mut parts = command.split_whitespace();
        let Some(program) = parts.next() else {
            continue;
        };
        let args: Vec<&str> = parts.collect();
        println!("[gate] pre-push: {command}");
        let status = Command::new(program)
            .args(&args)
            .status()
            .map_err(|error| {
                format!("cannot run `{command}`: {error}\n  check `.pengj-templates/gate.toml`")
            })?;
        if !status.success() {
            eprintln!("[gate] pre-push blocked: `{command}` failed");
            return Ok(1);
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_new_branch() {
        let refs = parse_refs("refs/heads/main\tabc123\trefs/heads/main\t0000000000000000000\n");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].local_ref, "refs/heads/main");
        assert_eq!(refs[0].local_sha, "abc123");
        assert_eq!(refs[0].remote_ref, "refs/heads/main");
    }

    #[test]
    fn skips_branch_deletion() {
        let refs = parse_refs(
            "(delete)\t0000000000000000000000000000000000000000\trefs/heads/old\tdeadbeef\n",
        );
        assert!(refs.is_empty(), "a deletion carries nothing to check");
    }

    #[test]
    fn skips_malformed_lines() {
        assert!(parse_refs("only-two\tfields\n").is_empty());
        assert!(parse_refs("\n\n").is_empty());
    }

    #[test]
    fn keeps_every_ref_of_a_multi_ref_push() {
        let stdin = "\
refs/heads/a\t1111\trefs/heads/a\t0000
refs/heads/b\t2222\trefs/heads/b\t3333
";
        let refs = parse_refs(stdin);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[1].remote_sha, "3333");
    }

    #[test]
    fn splits_a_command_into_program_and_args() {
        let mut parts = "just  gate-lang".split_whitespace();
        assert_eq!(parts.next(), Some("just"));
        assert_eq!(parts.collect::<Vec<_>>(), vec!["gate-lang"]);
    }

    #[test]
    fn parses_skip_commands_flag() {
        assert!(parse_skip_flag("1"));
        assert!(parse_skip_flag("true"));
        assert!(parse_skip_flag(" TRUE "));
        assert!(parse_skip_flag("yes"));
        assert!(!parse_skip_flag(""));
        assert!(!parse_skip_flag("0"));
        assert!(!parse_skip_flag("false"));
    }

    #[test]
    fn classifies_doc_only_and_code_paths() {
        assert!(is_doc_only_path("README.md"));
        assert!(is_doc_only_path("GLOSSARY.md"));
        assert!(is_doc_only_path("docs/adr/0032-linux.md"));
        assert!(is_doc_only_path(".agents/skills/foo/SKILL.md"));
        assert!(is_doc_only_path(".github/pull_request_template.md"));
        assert!(is_doc_only_path("LICENSE-MIT"));
        assert!(!is_doc_only_path("crates/gate/src/prepush.rs"));
        assert!(!is_doc_only_path("Cargo.toml"));
        assert!(!is_doc_only_path("templates/agent/AGENTS.md"));
        assert!(!is_doc_only_path(".github/workflows/ci.yml"));

        assert!(!has_code_changes(&[
            "README.md".to_string(),
            "docs/adr/0001.md".to_string(),
        ]));
        assert!(has_code_changes(&[
            "README.md".to_string(),
            "src/lib.rs".to_string(),
        ]));
    }

    #[test]
    fn detects_docs_only_and_mixed_pushes_in_git_repo() {
        let dir =
            std::env::temp_dir().join(format!("pengj-gate-prepush-diff-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");

        let run_git = |args: &[&str]| {
            let out = Command::new("git")
                .current_dir(&dir)
                .args(args)
                .output()
                .expect("run git");
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };

        run_git(&["init"]);
        run_git(&["config", "user.name", "tester"]);
        run_git(&["config", "user.email", "tester@example.com"]);
        std::fs::write(
            dir.join("README.md"),
            "init
",
        )
        .expect("write");
        run_git(&["add", "README.md"]);
        run_git(&["commit", "-m", "docs(init): 初始提交"]);
        let base_sha = run_git(&["rev-parse", "HEAD"]);

        // 纯文档提交
        std::fs::create_dir_all(dir.join("docs/adr")).expect("mkdir");
        std::fs::write(
            dir.join("docs/adr/0001.md"),
            "# ADR
",
        )
        .expect("write");
        run_git(&["add", "docs/adr/0001.md"]);
        run_git(&["commit", "-m", "docs(adr): 新增文档"]);
        let docs_sha = run_git(&["rev-parse", "HEAD"]);

        let docs_ref = vec![PushRef {
            local_ref: "refs/heads/main".to_string(),
            local_sha: docs_sha.clone(),
            remote_ref: "refs/heads/main".to_string(),
            remote_sha: base_sha.clone(),
        }];
        let files = changed_files_for_refs(&dir, &docs_ref).expect("should detect range");
        assert!(
            !has_code_changes(&files),
            "pure docs push should not have code changes"
        );

        // 混合代码提交
        std::fs::create_dir_all(dir.join("src")).expect("mkdir");
        std::fs::write(
            dir.join("src/lib.rs"),
            "pub fn f() {}
",
        )
        .expect("write");
        run_git(&["add", "src/lib.rs"]);
        run_git(&["commit", "-m", "feat(core): 新增函数"]);
        let code_sha = run_git(&["rev-parse", "HEAD"]);

        let code_ref = vec![PushRef {
            local_ref: "refs/heads/main".to_string(),
            local_sha: code_sha,
            remote_ref: "refs/heads/main".to_string(),
            remote_sha: docs_sha,
        }];
        let files = changed_files_for_refs(&dir, &code_ref).expect("should detect range");
        assert!(
            has_code_changes(&files),
            "code push should report code changes"
        );

        // 空引用列表保守返回 None
        assert!(changed_files_for_refs(&dir, &[]).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
