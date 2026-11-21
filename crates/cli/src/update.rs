//! CLI 自身的版本更新检查。
//!
//! 设计目标：**有新版就提示，但绝不拖慢任何一条命令**。
//!
//! - **异步**：`begin()` 只读本地缓存（几百字节，纯磁盘操作），需要联网时把检查动作
//!   **派生为一个独立的隐藏子进程**（`__update-check`），父进程立刻继续干活。
//!   子进程不占父进程的 stdin/stdout/stderr，父进程退出后它照样跑完并写回结果，
//!   于是检查结果永远服务于「下一次运行」；本次命令一秒都不等网络。
//!   这也是 npm/update-notifier、deno、uv 一类工具的共同做法。
//! - **节流**：默认 24 小时才真正联网一次（`PENGJ_UPDATE_CHECK_INTERVAL` 可改），
//!   所以命令用得多也不会反复发请求；提示本身同样最多每天一次。
//! - **安静**：提示只在 stderr 是终端时出现——管道、脚本、AI agent 调用看不到噪音，
//!   但检查照做，等人真的坐到终端前时缓存里已经有答案了；CI 环境完全不检查；
//!   `--no-update-check` 与 `PENGJ_NO_UPDATE_CHECK=1` 直接关掉全部动作。
//! - **失败即遗忘**：联网失败不重试、不报错、不写日志，等下一个周期再说。
//!
//! 状态文件落在平台数据目录（`%LOCALAPPDATA%\pengj-templates\update-check.json` /
//! `${XDG_DATA_HOME:-~/.local/share}/pengj-templates/update-check.json`）：
//!
//! ```json
//! {
//!   "last_check": 1763424000,        // 上次真正发起检查的时间（先写它，用来节流）
//!   "latest_version": "0.72.0",      // 上次检查到的最新版本
//!   "notified_version": "0.72.0",    // 上次向用户提示过的版本
//!   "notified_at": 1763424120        // 上次提示的时间（同一天不重复打扰）
//! }
//! ```

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 当前编译进二进制的版本号（release-please 写入 `Cargo.toml`）。
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "pengjiaxusz/pengj-templates";
const API_URL: &str = "https://api.github.com/repos/pengjiaxusz/pengj-templates/releases/latest";
const REDIRECT_URL: &str = "https://github.com/pengjiaxusz/pengj-templates/releases/latest";
const STATE_FILE: &str = "update-check.json";
const USER_AGENT: &str = concat!("pengj-templates-cli/", env!("CARGO_PKG_VERSION"));

/// 隐藏子进程的入口参数，也是父子进程之间唯一约定的字符串。
pub const CHILD_ARG: &str = "__update-check";
const CHILD_ENV: &str = "PENGJ_UPDATE_CHECK_CHILD";

/// 默认联网间隔：一天一次，频繁使用也不会反复发请求。
const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// 默认提示间隔：同一个提示一天最多出现一次。
const DEFAULT_NOTICE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// 联网总超时：后台子进程可以慢一点，但不能无限挂着。
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(8);

// ---------------------------------------------------------------------------
// 版本号
// ---------------------------------------------------------------------------

/// 简化的语义化版本号：够用就好，不引第三方 semver 依赖。
///
/// 支持 `0.72.0`、`v0.72.0`、`0.72.0-beta.1` 这类写法；比较规则与 semver 一致
/// （正式版 > 预发布，数字段按数值比，非数字段按字典序）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Vec<String>,
}

impl Version {
    /// 解析版本号；只容忍缺失的末段（`1.2` 视作 `1.2.0`），其余一律判为无效。
    pub fn parse(raw: &str) -> Option<Self> {
        let text = raw.trim();
        let text = text
            .strip_prefix('v')
            .or_else(|| text.strip_prefix('V'))
            .unwrap_or(text);
        // 构建元数据（`+abc`）不参与比较
        let text = text.split('+').next().unwrap_or(text);
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, pre),
            None => (text, ""),
        };
        let mut parts = core.split('.');
        let major = number(parts.next()?)?;
        let minor = match parts.next() {
            Some(text) => number(text)?,
            None => 0,
        };
        let patch = match parts.next() {
            Some(text) => number(text)?,
            None => 0,
        };
        if parts.next().is_some() {
            return None;
        }
        let pre = if pre.is_empty() {
            Vec::new()
        } else {
            pre.split('.').map(|part| part.to_string()).collect()
        };
        Some(Self {
            major,
            minor,
            patch,
            pre,
        })
    }
}

fn number(text: &str) -> Option<u64> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre.join("."))?;
        }
        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let core =
            (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch));
        if core != Ordering::Equal {
            return core;
        }
        match (self.pre.is_empty(), other.pre.is_empty()) {
            // 正式版高于同号预发布版
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                for (left, right) in self.pre.iter().zip(other.pre.iter()) {
                    let ordering = match (number(left), number(right)) {
                        (Some(a), Some(b)) => a.cmp(&b),
                        // 数字段优先级低于非数字段
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => left.cmp(right),
                    };
                    if ordering != Ordering::Equal {
                        return ordering;
                    }
                }
                self.pre.len().cmp(&other.pre.len())
            }
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ---------------------------------------------------------------------------
// 状态缓存
// ---------------------------------------------------------------------------

/// 更新检查的本地状态；字段全部可选，老版本写的文件 / 手工改坏的文件都能读。
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct State {
    #[serde(default)]
    last_check: Option<u64>,
    #[serde(default)]
    latest_version: Option<String>,
    #[serde(default)]
    notified_version: Option<String>,
    #[serde(default)]
    notified_at: Option<u64>,
}

fn state_path() -> Option<PathBuf> {
    pengj_core::platform_data_dir().map(|dir| dir.join(STATE_FILE))
}

fn load_state(path: &Path) -> State {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// 原子写：先写 `.tmp` 再改名，避免两个进程同时写时读到半截 JSON。
fn save_state(path: &Path, state: &State) {
    let Some(dir) = path.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let Ok(text) = serde_json::to_string_pretty(state) else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, text).is_err() {
        return;
    }
    if std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// 开关与节奏
// ---------------------------------------------------------------------------

fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// 布尔型环境变量：`1` / `true` / `yes` / `on` 视为开，`0` / `false` / `no` / `off` 视为关。
fn env_flag(name: &str) -> bool {
    match env_string(name) {
        Some(value) => !matches!(
            value.to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        ),
        None => false,
    }
}

/// 是否彻底关闭更新检查（含 CI 与隐藏子进程自身的防递归）。
fn is_disabled() -> bool {
    if env_flag("PENGJ_NO_UPDATE_CHECK") || env_flag("CI") || env_flag(CHILD_ENV) {
        return true;
    }
    matches!(
        env_string("PENGJ_UPDATE_CHECK_INTERVAL")
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("off") | Some("none") | Some("never")
    )
}

/// 联网间隔；`PENGJ_UPDATE_CHECK_INTERVAL` 支持纯秒数或 `12h` / `1d` 这类带单位写法。
fn check_interval() -> Duration {
    env_string("PENGJ_UPDATE_CHECK_INTERVAL")
        .as_deref()
        .and_then(parse_interval)
        .unwrap_or(DEFAULT_CHECK_INTERVAL)
}

fn notice_interval() -> Duration {
    env_string("PENGJ_NOTICE_INTERVAL")
        .as_deref()
        .and_then(parse_interval)
        .unwrap_or(DEFAULT_NOTICE_INTERVAL)
}

fn parse_interval(raw: &str) -> Option<Duration> {
    let text = raw.trim().to_ascii_lowercase();
    let (digits, factor) = match text.strip_suffix('h') {
        Some(rest) => (rest, 60 * 60),
        None => match text.strip_suffix('d') {
            Some(rest) => (rest, 24 * 60 * 60),
            None => match text.strip_suffix('m') {
                Some(rest) => (rest, 60),
                None => (text.as_str(), 1),
            },
        },
    };
    let seconds = number(digits.trim())?;
    Some(Duration::from_secs(seconds.saturating_mul(factor)))
}

/// 到点该联网了吗？`last_check` 缺失（首次运行）也算到点。
fn due(now: u64, last: Option<u64>, interval: Duration) -> bool {
    match last {
        None => true,
        Some(last) => now.saturating_sub(last) >= interval.as_secs(),
    }
}

/// 该提示了吗？发现「提示过的版本之外的更新版本」立刻提示，否则按天节流。
fn notice_due(state: &State, now: u64, latest: &Version, interval: Duration) -> bool {
    let latest_text = latest.to_string();
    if state.notified_version.as_deref() != Some(latest_text.as_str()) {
        return true;
    }
    due(now, state.notified_at, interval)
}

// ---------------------------------------------------------------------------
// 联网取版本
// ---------------------------------------------------------------------------

/// 联网代理：`self update` 要下载几十 MB 的便携包，公司网络/代理环境下
/// 读不到系统代理就直接失败，所以显式认这几个环境变量。
///
/// 只做「有就用」：解析不了就退化成直连，不因为代理配置写错而让命令不可用。
fn proxy_from_env() -> Option<ureq::Proxy> {
    let raw = [
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ]
    .iter()
    .find_map(|name| env_string(name))?;
    let url = if raw.contains("://") {
        raw
    } else {
        format!("http://{raw}")
    };
    ureq::Proxy::new(&url).ok()
}

pub(crate) fn agent(timeout: Duration) -> ureq::Agent {
    let mut builder = ureq::AgentBuilder::new()
        .user_agent(USER_AGENT)
        .timeout_connect(timeout.min(Duration::from_secs(3)))
        .timeout(timeout);
    if let Some(proxy) = proxy_from_env() {
        builder = builder.proxy(proxy);
    }
    builder.build()
}

/// 取最新正式版版本号。
///
/// 主路径是 GitHub API（结构化 JSON、一次请求）；限流或被墙时退到
/// `/releases/latest` 的 302 跳转（从最终 URL 的 `/tag/vX.Y.Z` 里读版本，
/// 不占 API 配额）。`PENGJ_UPDATE_API` 可以把主路径指到镜像或自建接口。
fn fetch_latest(timeout: Duration) -> Result<Version, String> {
    if let Some(url) = env_string("PENGJ_UPDATE_API") {
        return fetch_from_api(&url, timeout);
    }
    match fetch_from_api(API_URL, timeout) {
        Ok(version) => Ok(version),
        Err(api_error) => fetch_from_redirect(REDIRECT_URL, timeout).map_err(|_| api_error),
    }
}

/// 供 `self update` 复用：同步取最新发布版版本号。
pub fn latest_version(timeout: Duration) -> anyhow::Result<Version> {
    fetch_latest(timeout).map_err(|error| anyhow::anyhow!("{error}"))
}

/// 升级完成后把缓存对齐到实际装上的版本。
///
/// 不写这一笔的话，下一次运行还是会照着旧缓存提示「有新版本可用」，
/// 明明刚升过却一直在催。
pub fn record_installed(version: &Version) {
    let Some(path) = state_path() else {
        return;
    };
    let mut state = load_state(&path);
    state.latest_version = Some(version.to_string());
    state.notified_version = Some(version.to_string());
    state.notified_at = Some(unix_now());
    save_state(&path, &state);
}

fn fetch_from_api(url: &str, timeout: Duration) -> Result<Version, String> {
    let response = agent(timeout)
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| format!("请求失败: {error}"))?;
    let body = response
        .into_string()
        .map_err(|error| format!("读取响应失败: {error}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&body).map_err(|error| format!("解析响应失败: {error}"))?;
    let tag = value
        .get("tag_name")
        .and_then(|tag| tag.as_str())
        .ok_or_else(|| "响应里没有 tag_name 字段".to_string())?;
    Version::parse(tag).ok_or_else(|| format!("无法解析版本号: {tag}"))
}

fn fetch_from_redirect(url: &str, timeout: Duration) -> Result<Version, String> {
    let response = agent(timeout)
        .get(url)
        .call()
        .map_err(|error| format!("请求失败: {error}"))?;
    let final_url = response.get_url().to_string();
    version_from_release_url(&final_url)
        .ok_or_else(|| format!("跳转后的地址里没有版本号: {final_url}"))
}

/// 从 `https://github.com/<owner>/<repo>/releases/tag/v0.72.0` 里取版本号。
fn version_from_release_url(url: &str) -> Option<Version> {
    let tail = url.split("/releases/tag/").nth(1)?;
    let tail = tail.split(['?', '#']).next()?;
    let tail = tail.trim_end_matches('/');
    let segment = tail.rsplit('/').next()?;
    Version::parse(segment)
}

// ---------------------------------------------------------------------------
// 父子进程协作
// ---------------------------------------------------------------------------

/// 本次进程的提示状态；`begin()` 填，`finish()` 取走并打印。
struct Session {
    notice: Option<String>,
    latest: Option<Version>,
    path: Option<PathBuf>,
    now: u64,
}

static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();

fn session() -> MutexGuard<'static, Option<Session>> {
    SESSION
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 进程启动时调用一次：读缓存、按需派生后台检查、算好提示文案。
///
/// 永远不阻塞：这里没有任何网络调用。
pub fn begin(disabled_by_flag: bool) {
    if disabled_by_flag || is_disabled() {
        return;
    }
    let Some(path) = state_path() else { return };
    let now = unix_now();
    let mut state = load_state(&path);

    if due(now, state.last_check, check_interval()) {
        // 先落 last_check 再派生：同时开好几个进程时，后来者不会重复联网
        state.last_check = Some(now);
        save_state(&path, &state);
        spawn_child();
    }

    let current = Version::parse(CURRENT_VERSION);
    let latest = state
        .latest_version
        .as_deref()
        .and_then(Version::parse)
        .filter(|latest| current.as_ref().is_some_and(|current| latest > current));

    // 提示只给「人在终端里」看：管道、重定向、agent 调用一律不打扰，
    // 但检查照做——这样等真的有人坐在终端前时，缓存里已经有答案了。
    let quiet = !std::io::stderr().is_terminal();
    let notice = match (&latest, quiet) {
        (Some(latest), false) => {
            if notice_due(&state, now, latest, notice_interval()) {
                current
                    .as_ref()
                    .map(|current| render_notice(latest, current))
            } else {
                None
            }
        }
        _ => None,
    };

    *session() = Some(Session {
        notice,
        latest,
        path: Some(path),
        now,
    });
}

/// 命令跑完后调用：打印「有新版本」提示，并记下「已提示」避免天天打扰。
///
/// 幂等：重复调用只生效一次（`exit()` 与 `main` 末尾都会调用）。
pub fn finish() {
    let Some(session) = session().take() else {
        return;
    };
    let Some(text) = session.notice else { return };
    eprintln!("{text}");
    if let (Some(path), Some(latest)) = (session.path, session.latest) {
        let mut state = load_state(&path);
        state.notified_version = Some(latest.to_string());
        state.notified_at = Some(session.now);
        save_state(&path, &state);
    }
}

/// 先补打印更新提示，再按给定退出码退出。
///
/// 命令里凡是要用 `std::process::exit` 的地方都改走这里，否则提示会随提前退出丢掉。
pub fn exit(code: i32) -> ! {
    finish();
    std::process::exit(code);
}

/// 派生后台检查子进程：stdio 全部丢弃，父进程退出后它仍然继续跑完。
fn spawn_child() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let mut command = std::process::Command::new(exe);
    command
        .arg(CHILD_ARG)
        .env(CHILD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW：不弹黑框。子进程不随父进程退出而终止，正好用来跑检查。
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = command.spawn();
}

/// 隐藏子命令 `__update-check` 的入口：联网、写回缓存、静默退出。
pub fn run_background() -> i32 {
    let Some(path) = state_path() else {
        return 0;
    };
    let latest = fetch_latest(DEFAULT_TIMEOUT).ok();
    // 取完版本再读状态：把「父子进程同时写」的竞争窗口缩到一次读-改-写，
    // 父进程刚记下的 notified_* 不会被这半秒前的旧快照覆盖掉。
    let mut state = load_state(&path);
    state.last_check = Some(unix_now());
    if let Some(latest) = latest {
        state.latest_version = Some(latest.to_string());
    }
    save_state(&path, &state);
    0
}

// ---------------------------------------------------------------------------
// 显式检查命令
// ---------------------------------------------------------------------------

pub struct CheckOptions {
    pub json: bool,
    /// 联网超时（秒），缺省 8 秒。
    pub timeout: Option<u64>,
}

/// `check-update` 的实盘：同步联网，结果同时写回缓存供后续运行复用。
///
/// 返回 `Ok(true)` 表示有新版本（由调用方决定退出码：检查本身不算错误）。
pub fn run_check(options: &CheckOptions) -> anyhow::Result<bool> {
    let timeout = Duration::from_secs(options.timeout.unwrap_or(DEFAULT_TIMEOUT.as_secs()).max(1));
    let current = Version::parse(CURRENT_VERSION);

    let outcome = fetch_latest(timeout);
    if let Some(path) = state_path() {
        let mut state = load_state(&path);
        state.last_check = Some(unix_now());
        if let Ok(latest) = &outcome {
            state.latest_version = Some(latest.to_string());
        }
        save_state(&path, &state);
    }

    let latest = outcome.as_ref().ok().cloned();
    let error = outcome.as_ref().err().cloned();
    let update_available = match (&current, &latest) {
        (Some(current), Some(latest)) => latest > current,
        _ => false,
    };

    if options.json {
        let payload = serde_json::json!({
            "current": CURRENT_VERSION,
            "latest": latest.as_ref().map(ToString::to_string),
            "updateAvailable": update_available,
            "error": error,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("当前版本: {CURRENT_VERSION}");
        match &latest {
            Some(latest) => println!("最新版本: {latest}"),
            None => println!("最新版本: 未知（检查失败，可稍后重试）"),
        }
        if update_available {
            println!("状态: 有新版本可用");
            println!("{}", upgrade_hint());
        } else if latest.is_some() {
            println!("状态: 已是最新");
        }
    }

    if let Some(error) = error {
        anyhow::bail!("检查更新失败: {error}");
    }
    Ok(update_available)
}

// ---------------------------------------------------------------------------
// 文案
// ---------------------------------------------------------------------------

/// 升级命令：自升级已经内建，不再需要 curl 一条龙。
///
/// 保留一行回退地址：装着的还是「没有 `self update` 的旧版本」时，
/// 只能靠引导脚本把自己换成新版，这一步之后就都用命令了。
fn upgrade_hint() -> String {
    let legacy = if cfg!(windows) {
        format!("irm https://github.com/{REPO}/releases/latest/download/install.ps1 | iex")
    } else {
        format!("curl -fsSL https://github.com/{REPO}/releases/latest/download/install.sh | sh")
    };
    format!("升级: pengj-templates-cli self update\n  旧版本不认这个命令时: {legacy}")
}

fn render_notice(latest: &Version, current: &Version) -> String {
    format!(
        "[pengj-templates-cli] 新版本 {latest} 可用（当前 {current}）\n  {}\n  不再提示: 加 --no-update-check，或设 PENGJ_NO_UPDATE_CHECK=1",
        upgrade_hint()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(raw: &str) -> Version {
        Version::parse(raw).unwrap_or_else(|| panic!("无法解析版本号 {raw}"))
    }

    #[test]
    fn 解析常见版本号写法() {
        assert_eq!(version("0.71.0").to_string(), "0.71.0");
        assert_eq!(version("v0.72.0").to_string(), "0.72.0");
        assert_eq!(version(" 1.2 ").to_string(), "1.2.0");
        assert_eq!(version("0.72.0-beta.1").to_string(), "0.72.0-beta.1");
        assert_eq!(version("0.72.0+build.5").to_string(), "0.72.0");
        assert!(Version::parse("").is_none());
        assert!(Version::parse("abc").is_none());
        assert!(Version::parse("1.2.3.4").is_none());
        assert!(Version::parse("1.x.0").is_none());
    }

    #[test]
    fn 版本号按语义化比较() {
        assert!(version("0.72.0") > version("0.71.9"));
        assert!(version("0.10.0") > version("0.9.9"));
        assert!(version("1.0.0") > version("0.99.99"));
        // 正式版高于预发布版，预发布之间按段比较
        assert!(version("0.71.0") > version("0.71.0-beta.2"));
        assert!(version("0.71.0-beta.2") > version("0.71.0-beta.1"));
        // 预发布字段多一个段，优先级更低（semver：beta < beta.1 < rc.1）
        assert!(version("0.71.0-beta.1") > version("0.71.0-beta"));
        assert!(version("0.71.0-rc.1") > version("0.71.0-beta.9"));
        assert_eq!(version("v0.71.0"), version("0.71.0"));
    }

    #[test]
    fn 检查节奏按间隔节流() {
        let day = DEFAULT_CHECK_INTERVAL;
        let now = 10 * day.as_secs();
        assert!(due(now, None, day), "首次运行应当检查");
        assert!(!due(now, Some(now - 60), day), "刚查过不该再查");
        assert!(due(now, Some(now - day.as_secs()), day), "到点应当再查");
        assert!(due(now, Some(1), day), "隔了很久也应当再查");
    }

    #[test]
    fn 提示对新版本立即出现且按天节流() {
        let now = 1_000_000u64;
        let day = DEFAULT_NOTICE_INTERVAL;
        let latest = version("0.72.0");
        let state = State::default();
        assert!(notice_due(&state, now, &latest, day), "首次发现应当提示");

        let notified = State {
            notified_version: Some("0.72.0".to_string()),
            notified_at: Some(now - 60),
            ..State::default()
        };
        assert!(
            !notice_due(&notified, now, &latest, day),
            "同一天不重复提示"
        );
        assert!(
            notice_due(&notified, now + day.as_secs(), &latest, day),
            "过了一天再提示一次"
        );

        let newer = version("0.73.0");
        assert!(
            notice_due(&notified, now, &newer, day),
            "换了个新版本要立刻提示"
        );
    }

    #[test]
    fn 间隔配置支持秒与带单位写法() {
        assert_eq!(parse_interval("3600"), Some(Duration::from_secs(3600)));
        assert_eq!(parse_interval("12h"), Some(Duration::from_secs(12 * 3600)));
        assert_eq!(parse_interval("1d"), Some(Duration::from_secs(24 * 3600)));
        assert_eq!(parse_interval("30m"), Some(Duration::from_secs(30 * 60)));
        assert_eq!(parse_interval("off"), None);
        assert_eq!(parse_interval("abc"), None);
    }

    #[test]
    fn 从跳转地址里读版本号() {
        assert_eq!(
            version_from_release_url(
                "https://github.com/pengjiaxusz/pengj-templates/releases/tag/v0.72.0"
            ),
            Some(version("0.72.0"))
        );
        assert_eq!(
            version_from_release_url("https://github.com/x/y/releases/tag/v0.72.0?expanded=true"),
            Some(version("0.72.0"))
        );
        assert_eq!(
            version_from_release_url("https://github.com/x/y/releases"),
            None
        );
    }

    #[test]
    fn 状态文件缺字段也能读() {
        let state: State =
            serde_json::from_str(r#"{"latest_version":"0.72.0"}"#).unwrap_or_default();
        assert_eq!(state.latest_version.as_deref(), Some("0.72.0"));
        assert_eq!(state.last_check, None);
    }

    #[test]
    fn 状态文件可往返读写且容忍损坏内容() {
        let dir = std::env::temp_dir().join(format!("pengj-update-test-{}", std::process::id()));
        let path = dir.join(STATE_FILE);
        let state = State {
            last_check: Some(7),
            latest_version: Some("0.72.0".to_string()),
            ..State::default()
        };
        save_state(&path, &state);
        let loaded = load_state(&path);
        assert_eq!(loaded.last_check, Some(7));
        assert_eq!(loaded.latest_version.as_deref(), Some("0.72.0"));

        // 半截 / 手改坏的文件不能让检查流程崩掉，直接当空状态处理
        if let Err(error) = std::fs::write(&path, "{ not json") {
            panic!("写入损坏文件失败: {error}");
        }
        let broken = load_state(&path);
        assert_eq!(broken.last_check, None);
        assert_eq!(broken.latest_version, None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 提示文案给出新版本与升级命令() {
        let text = render_notice(&version("0.72.0"), &version("0.71.0"));
        assert!(text.contains("0.72.0"), "缺少新版本号: {text}");
        assert!(text.contains("0.71.0"), "缺少当前版本号: {text}");
        assert!(
            text.contains("releases/latest/download/install"),
            "缺少升级命令: {text}"
        );
        assert!(
            text.contains("PENGJ_NO_UPDATE_CHECK=1"),
            "缺少关闭提示的办法: {text}"
        );
    }
}
