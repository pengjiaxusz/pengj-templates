//! 兜底脚本与 CLI 的裁决方向必须一致。
//!
//! `.githooks/gate-fallback.sh` 是没装 `pengj-templates-cli` 的机器上的降级检查。
//! 它有一条不变式：**永远不比 CLI 严格**——脚本拒掉的提交，CLI 也一定拒；脚本放过的
//! 提交，CLI 可能还拒（那由服务端 CI 兜底）。这条不变式由本测试守住：逐条样例消息
//! 同时跑脚本与库里的规则，脚本一旦误杀合规提交，测试立刻红。
//!
//! 脚本是 POSIX sh，所以测试需要一个 `sh`：找不到就跳过（Windows 上没装 Git Bash 时）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use pengj_core::Templates;

/// 找一个能用的 `sh`；找不到返回 `None`（测试跳过）。
fn find_sh() -> Option<String> {
    for candidate in ["sh", "bash"] {
        let ok = Command::new(candidate)
            .arg("-c")
            .arg("exit 0")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if ok {
            return Some(candidate.to_string());
        }
    }
    None
}

/// 模板根目录：`crates/gate/` 上两级就是仓库根。
fn templates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("templates")
}

/// 渲染一个只带 `gate` 层的项目，返回项目根目录。
///
/// `commit_zh` 决定 `.gate.toml` 里 `commit.subject_lang` 的取值，
/// 也就是提交主题语言规则的方向。
fn render_gate_project(tag: &str, commit_zh: bool) -> PathBuf {
    let parent = std::env::temp_dir().join(format!("pengj-fallback-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&parent);
    std::fs::create_dir_all(&parent).expect("temp dir");

    let templates = Templates::new(templates_root());
    let mut options = BTreeMap::new();
    options.insert("skill_lang".to_string(), serde_json::json!("zh"));
    options.insert("chinese_programming".to_string(), serde_json::json!(true));
    options.insert("commit_zh".to_string(), serde_json::json!(commit_zh));

    let layers = vec!["common".to_string(), "gate".to_string()];
    pengj_core::generate(&templates, "fallback-case", &layers, options, &parent)
        .expect("generate the gate layer");

    let project = parent.join("fallback-case");

    // 这些样例只用 `(cli)` 当占位 scope，聚焦的是语言与格式规则。模板默认
    // `scopes = []` 会拒绝一切 scope，这里显式放行，免得 scope 规则变成噪声。
    let config_path = project.join(".gate.toml");
    let config = std::fs::read_to_string(&config_path).expect("read .gate.toml");
    let relaxed = config.replace("commit.scopes = []", "commit.scopes = [\"*\"]");
    assert_ne!(
        config, relaxed,
        "the rendered config must carry commit.scopes"
    );
    std::fs::write(&config_path, relaxed).expect("write .gate.toml");

    project
}

/// 跑兜底脚本：返回它是否放行这条提交。
fn fallback_accepts(sh: &str, project: &Path, message: &str) -> bool {
    let script = project.join(".githooks").join("gate-fallback.sh");
    let msg_file = project.join("COMMIT_EDITMSG");
    std::fs::write(&msg_file, message).expect("write the commit message");

    let output = Command::new(sh)
        .arg(&script)
        .arg(&msg_file)
        .current_dir(project)
        .output()
        .expect("run the fallback script");
    output.status.success()
}

/// 跑库里的规则：返回它是否放行这条提交。
fn cli_accepts(project: &Path, message: &str) -> bool {
    let (config, _) = pengj_gate::config::load(project).expect("load .gate.toml");
    let rules = pengj_gate::commit::Rules::from_config(&config);
    pengj_gate::commit::validate(message, &rules).is_empty()
}

/// 中文项目的样例：`(提交标题, 期望被拒)`。
const ZH_CASES: &[(&str, bool)] = &[
    ("feat(cli): 新增参数解析支持", false),
    ("fix: 修复空指针", false),
    ("feat(cli)!: 新增参数解析支持", false),
    ("chore(main): 发布版本", false),
    ("docs: 更新说明文档", false),
    ("Merge pull request #12 from upstream", false),
    ("feat(cli): add new flags", true),
    ("feat(cli): 新", true),
    ("foo(cli): 新增参数解析", true),
    ("feat(cli) 新增参数解析", true),
    ("feat(cli):", true),
    ("feat(cli): 新增参数解析.", true),
    ("feat(cli): 新增参数解析 ", true),
];

/// 英文项目的样例：`(提交标题, 期望被拒)`。
const EN_CASES: &[(&str, bool)] = &[
    ("feat(cli): add new flags", false),
    ("fix: repair the null pointer", false),
    ("feat(cli): add 'quoted term' support", false),
    ("feat(cli): 新增参数解析支持", true),
    ("feat(cli): 中文字符串支持功能上线", true),
];

/// 兜底脚本拒掉的提交，CLI 也必须拒：脚本不能比 CLI 严格。
fn assert_fallback_never_stricter(sh: &str, project: &Path, cases: &[(&str, bool)], tag: &str) {
    for (message, expected_rejected) in cases {
        let fallback = fallback_accepts(sh, project, message);
        let cli = cli_accepts(project, message);
        assert_eq!(
            !fallback, *expected_rejected,
            "{tag}: the fallback script has the wrong verdict for `{message}`"
        );
        if fallback {
            assert!(
                cli,
                "{tag}: the fallback script rejected `{message}`, but the CLI accepts it"
            );
        }
    }
}

#[test]
fn the_fallback_script_is_never_stricter_than_the_cli() {
    let Some(sh) = find_sh() else {
        eprintln!("skipped: no POSIX shell available");
        return;
    };
    let project = render_gate_project("zh-cases", true);
    assert_fallback_never_stricter(&sh, &project, ZH_CASES, "zh");

    let _ = std::fs::remove_dir_all(project.parent().unwrap());
}

#[test]
fn the_fallback_script_follows_the_commit_language_axis() {
    let Some(sh) = find_sh() else {
        eprintln!("skipped: no POSIX shell available");
        return;
    };
    let project = render_gate_project("en-cases", false);
    assert_fallback_never_stricter(&sh, &project, EN_CASES, "en");

    let _ = std::fs::remove_dir_all(project.parent().unwrap());
}

#[test]
fn the_fallback_script_reads_its_thresholds_from_the_config() {
    let Some(sh) = find_sh() else {
        eprintln!("skipped: no POSIX shell available");
        return;
    };
    let project = render_gate_project("zh-thresholds", true);

    // `.gate.toml` 里把最少汉字数提到 4，一个汉字的标题就该被拒。
    let config_path = project.join(".gate.toml");
    let config = std::fs::read_to_string(&config_path).expect("read .gate.toml");
    let raised = config.replace("commit.subject_min_han = 2", "commit.subject_min_han = 4");
    assert_ne!(
        config, raised,
        "the rendered config must carry the threshold"
    );
    std::fs::write(&config_path, raised).expect("write .gate.toml");

    assert!(
        !fallback_accepts(&sh, &project, "feat(cli): 新增"),
        "the fallback script must follow commit.subject_min_han from .gate.toml"
    );
    assert!(
        fallback_accepts(&sh, &project, "feat(cli): 新增参数解析支持"),
        "a subject with enough Han characters must still pass"
    );

    let _ = std::fs::remove_dir_all(project.parent().unwrap());
}
