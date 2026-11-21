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
//! 内置检查只有两项，都是「CI 会跑、本地最容易漏」的：`cargo fmt --check`（见
//! [`check_fmt`]）与代码语言守卫（在 `cli` 里编排，因为它要读 `gate.toml` 的模式）。
//! 项目自己的重型检查（clippy 之类）由 `prepush.commands` 声明、按序执行——钩子不替
//! 项目决定跑什么，只保证「声明了就一定跑」。

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
}
