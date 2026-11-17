//! End-to-end coverage for the server-side commit check.
//!
//! `check_commits` is what a CI run calls: it walks a range of commits and
//! applies the same rules the `commit-msg` hook applies. The tests drive it
//! against a real repository so the git plumbing is exercised too.

use pengj_gate::commit;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A throwaway git repository with the gate's own configuration in place.
struct Repo {
    dir: PathBuf,
}

impl Repo {
    fn new(name: &str) -> Repo {
        let dir =
            std::env::temp_dir().join(format!("pengj-check-commits-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create the temporary repository");
        let repo = Repo { dir };
        repo.git(&["init", "-q", "-b", "main"]);
        repo.git(&["config", "user.name", "gate test"]);
        repo.git(&["config", "user.email", "gate@example.com"]);
        // The rules the generated project ships with: Chinese subjects. The
        // wildcard keeps these language-focused tests free of scope noise.
        repo.write_config(
            "[lang]\nmode = \"chinese\"\n\n[commit]\nsubject_lang = \"zh\"\nsubject_min_han = 2\nscopes = [\"*\"]\n",
        );
        repo
    }

    fn write_config(&self, body: &str) {
        std::fs::write(self.dir.join(".gate.toml"), body).expect("write .gate.toml");
    }

    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .expect("run git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn commit(&self, message: &str) -> String {
        std::fs::write(self.dir.join("file.txt"), message).expect("write the tracked file");
        self.git(&["add", "file.txt"]);
        self.git(&["commit", "-q", "--allow-empty-message", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    /// Run the check inside the repository. No global state is touched, so
    /// the tests can run in parallel.
    fn check(&self, base: Option<&str>) -> Result<commit::CommitCheckReport, String> {
        commit::check_commits_in(&self.dir, base, None)
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn gate_toml_exists(dir: &Path) -> bool {
    dir.join(".gate.toml").is_file()
}

#[test]
fn accepts_a_range_of_valid_messages() {
    let repo = Repo::new("valid");
    assert!(gate_toml_exists(&repo.dir));
    let base = repo.commit("chore(init): 初始化项目骨架");
    repo.commit("feat(core): 新增解析能力");

    let report = repo.check(Some(&base)).expect("the range is reachable");
    assert_eq!(report.checked, 1);
    assert!(
        report.rejected.is_empty(),
        "unexpected rejections: {:?}",
        report.rejected
    );
}

#[test]
fn rejects_a_scope_when_no_whitelist_is_configured() {
    let repo = Repo::new("no-scope");
    // 空列表与省略 key 同为 fail-closed：任何 scope 都不接受。
    repo.write_config(
        "[lang]\nmode = \"chinese\"\n\n[commit]\nsubject_lang = \"zh\"\nsubject_min_han = 2\nscopes = []\n",
    );
    let base = repo.commit("chore: 初始化项目骨架");
    repo.commit("feat(core): 新增解析能力");

    let report = repo.check(Some(&base)).expect("the range is reachable");
    assert_eq!(report.checked, 1);
    assert_eq!(report.rejected.len(), 1, "the scope must be rejected");
    assert!(
        report.rejected[0]
            .issues
            .iter()
            .any(|issue| issue.contains("scope-enum")),
        "the scope rule must be reported: {:?}",
        report.rejected[0].issues
    );
}

#[test]
fn rejects_an_english_subject_in_the_range() {
    let repo = Repo::new("english");
    let base = repo.commit("chore(init): 初始化项目骨架");
    repo.commit("feat(core): add new feature");

    let report = repo.check(Some(&base)).expect("the range is reachable");
    assert_eq!(report.checked, 1);
    assert_eq!(
        report.rejected.len(),
        1,
        "the English subject must be rejected"
    );
    let rejected = &report.rejected[0];
    assert_eq!(rejected.subject, "feat(core): add new feature");
    assert!(
        rejected
            .issues
            .iter()
            .any(|issue| issue.contains("subject-language")),
        "the language rule must be reported: {:?}",
        rejected.issues
    );
}

#[test]
fn an_empty_range_is_not_a_failure() {
    let repo = Repo::new("empty");
    let base = repo.commit("chore(init): 初始化项目骨架");

    let report = repo.check(Some(&base)).expect("the range is reachable");
    assert_eq!(report.checked, 0);
    assert!(report.rejected.is_empty());
}

#[test]
fn reports_an_unreachable_base_ref() {
    let repo = Repo::new("unreachable");
    repo.commit("chore(init): 初始化项目骨架");

    let error = repo
        .check(Some("HEAD~99"))
        .expect_err("an unreachable base must fail");
    assert!(
        error.contains("does not exist"),
        "the error must name the problem: {error}"
    );
}
