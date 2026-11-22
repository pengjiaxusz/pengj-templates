//! `self update`：把装在机器上的这个工具本身换成新版本。
//!
//! 与 `update` 的分工是刻意分成两层的：
//!
//! - `self update` 换的是**生产工具**：安装根（`%LOCALAPPDATA%\pengj-templates`
//!   或 `$XDG_DATA_HOME/pengj-templates`）下的 `bin/pengj-templates-cli` 与
//!   `templates/`。两者必须同版本一起换 —— engine 是运行时读模板的，只换可执
//!   行文件会得到「新程序 + 旧模板」的错配。
//! - `update` 换的是**被生产物**：已纳管项目里的受管产物。工具升级后项目不会
//!   自动跟进，要同步得自己跑 `update`（命令结束时会提醒这一句）。
//!
//! 两条通道：
//!
//! - 默认走 GitHub Releases：查最新版 → 下载便携包（CLI + templates）→ 解包 →
//!   原地替换。装着的已是最新就一个字节都不写。
//! - `--from-source` 走本机构建：`cargo build --release` 后用工作区里的产物覆
//!   盖安装。这条路拿得到**未发布**的改动，是日常开发「让机器上的 CLI 用上刚
//!   写的代码」的正解，而发布包永远慢一步。
//!
//! 三个平台细节值得记一笔：
//!
//! - Windows 上运行中的 exe 不能覆盖，但**可以改名**：先挪成 `.old-<pid>`，
//!   再写新文件，写失败就把旧的挪回来。`tools/install.ps1` 用的是同一招。
//! - PATH 上可能同时躺着好几份副本（scoop 的 `.cargo/bin`、`~/.cargo/bin`、
//!   安装目录），排在前面的会抢占。装完顺手把它们一起刷成新版本，否则会出现
//!   「升了，但敲命令还是旧的」。
//! - 便携包解出来带一层 `pengj-templates/` 前缀，按目录找根，不认死结构。

use std::env;
use std::fs;
use std::io;
#[cfg(any(windows, test))]
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context};

use crate::update::{self, Version};

const REPO: &str = "pengjiaxusz/pengj-templates";
/// 便携包里的顶层目录名（release.yml 打包时固定这一层）。
const BUNDLE_DIR: &str = "pengj-templates";
#[cfg(windows)]
const EXE_NAME: &str = "pengj-templates-cli.exe";
#[cfg(not(windows))]
const EXE_NAME: &str = "pengj-templates-cli";

/// 查版本用的超时：和被动检查保持一致，失败得快一点。
const VERSION_TIMEOUT: Duration = Duration::from_secs(8);
/// 下载超时：便携包十几 MB，弱网也得给够。
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);

pub struct Options {
    /// 从当前工作区源码构建后安装，而不是下载发布包。
    pub from_source: bool,
    /// 指定版本（回滚用）；缺省取最新发布版。
    pub version: Option<String>,
    /// 同版本也重装、允许降级。
    pub force: bool,
    /// 只打印计划，不落盘。
    pub dry_run: bool,
    /// 不刷新 PATH 上的其它副本。
    pub no_path_refresh: bool,
    /// `--from-source` 的工作区根（缺省从当前目录向上找）。
    pub workspace: Option<PathBuf>,
    /// 查版本的联网超时（秒）。
    pub timeout: Option<u64>,
}

/// 安装布局：`<root>/bin/<exe>` + `<root>/templates`，与两个安装脚本一致。
struct Layout {
    root: PathBuf,
    exe: PathBuf,
    templates: PathBuf,
}

impl Layout {
    fn resolve() -> anyhow::Result<Self> {
        let root = install_root()?;
        Ok(Self::at(root))
    }

    fn at(root: PathBuf) -> Self {
        Self {
            exe: root.join("bin").join(EXE_NAME),
            templates: root.join("templates"),
            root,
        }
    }
}

/// 安装根：与 engine 找模板用的是同一个位置，测试与 CI 可用 `PENGJ_INSTALL_ROOT` 改指。
fn install_root() -> anyhow::Result<PathBuf> {
    if let Some(dir) = env::var_os("PENGJ_INSTALL_ROOT") {
        return Ok(PathBuf::from(dir));
    }
    pengj_core::platform_data_dir().ok_or_else(|| {
        anyhow!(
            "无法推导安装根目录（缺少 LOCALAPPDATA / XDG_DATA_HOME），可用 PENGJ_INSTALL_ROOT 指定"
        )
    })
}

pub fn run(options: &Options) -> anyhow::Result<()> {
    let layout = Layout::resolve()?;
    let current = Version::parse(update::CURRENT_VERSION)
        .ok_or_else(|| anyhow!("内置版本号无法解析: {}", update::CURRENT_VERSION))?;
    if options.from_source {
        return from_source(&layout, options);
    }
    from_release(&layout, &current, options)
}

// ---------------------------------------------------------------------------
// 通道一：GitHub Releases
// ---------------------------------------------------------------------------

fn from_release(layout: &Layout, current: &Version, options: &Options) -> anyhow::Result<()> {
    let timeout = Duration::from_secs(options.timeout.unwrap_or(VERSION_TIMEOUT.as_secs()).max(1));
    let target = match &options.version {
        Some(raw) => Version::parse(raw).ok_or_else(|| anyhow!("无法解析版本号: {raw}"))?,
        None => update::latest_version(timeout)
            .context("查询最新版本失败（网络或代理？也可用 --version 指定）")?,
    };

    println!("当前版本: {current}");
    println!("目标版本: {target}");

    if target == *current && !options.force {
        println!("已是最新，无需升级（--force 可强制重装同一版本）。");
        return Ok(());
    }
    if target < *current && !options.force {
        println!("目标版本低于当前版本，默认不降级（确实要回滚请加 --force）。");
        return Ok(());
    }

    let (os, arch) = platform()?;
    let asset = bundle_asset_name(os, arch);
    let url = format!("https://github.com/{REPO}/releases/download/v{target}/{asset}");

    println!("安装根: {}", layout.root.display());
    println!("下载: {url}");
    if options.dry_run {
        println!(
            "[dry-run] 将替换 {} 与 {}",
            layout.exe.display(),
            layout.templates.display()
        );
        return Ok(());
    }

    let temp = TempDir::new("pengj-self-update")?;
    let archive = temp.path().join(&asset);
    let size = download(&url, &archive)?;
    println!("已下载: {:.1} MiB", size as f64 / (1024.0 * 1024.0));

    let staging = temp.path().join("unpacked");
    fs::create_dir_all(&staging)?;
    extract(&archive, &staging)?;
    let bundle = bundle_root(&staging);

    let source_exe = bundle.join(EXE_NAME);
    let source_templates = bundle.join("templates");
    if !source_exe.is_file() {
        bail!("便携包里没有 {EXE_NAME}，包可能损坏");
    }
    if !source_templates.is_dir() {
        bail!("便携包里没有 templates/，拒绝装半套（engine 运行时依赖它）");
    }

    apply(layout, &source_exe, &source_templates, options)?;
    verify(layout, Some(&target))?;
    update::record_installed(&target);
    closing_hint();
    Ok(())
}

/// 平台与架构标签，必须和 release.yml 矩阵里的 `os` / `arch` 对齐。
fn platform() -> anyhow::Result<(&'static str, &'static str)> {
    let os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        bail!("暂不支持当前操作系统，请用 tools/install 脚本手工安装");
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        bail!("暂不支持当前架构，请用 tools/install 脚本手工安装");
    };
    Ok((os, arch))
}

fn bundle_asset_name(os: &str, arch: &str) -> String {
    let ext = if cfg!(windows) { "zip" } else { "tar.gz" };
    format!("pengj-templates-cli-{os}-{arch}.{ext}")
}

fn download(url: &str, dest: &Path) -> anyhow::Result<u64> {
    let agent = update::agent(DOWNLOAD_TIMEOUT);
    let response = agent
        .get(url)
        .call()
        .map_err(|error| anyhow!("下载失败: {error}"))?;
    let mut reader = response.into_reader();
    let mut file =
        fs::File::create(dest).with_context(|| format!("无法创建临时文件 {}", dest.display()))?;
    let size = io::copy(&mut reader, &mut file).context("写入下载文件失败")?;
    if size == 0 {
        bail!("下载到的文件是空的，请稍后重试");
    }
    Ok(size)
}

#[cfg(windows)]
fn extract(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    let file =
        fs::File::open(archive).with_context(|| format!("无法打开 {}", archive.display()))?;
    let mut zip = zip::ZipArchive::new(file).context("便携包不是有效的 zip")?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        // 压缩包里的路径只是数据，不能当指令用：绝对路径与 `..` 一律丢弃
        let Some(relative) = safe_relative(Path::new(entry.name())) else {
            continue;
        };
        let out = dest.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut target = fs::File::create(&out)?;
        io::copy(&mut entry, &mut target)?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn extract(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    let file =
        fs::File::open(archive).with_context(|| format!("无法打开 {}", archive.display()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    // tar 的 unpack 自己会跳过越界条目，解不到 dest 之外
    archive.unpack(dest).context("解包 tar.gz 失败")?;
    Ok(())
}

/// 只接受纯相对路径；`..`、盘符、根路径一律判为不可信。
// 调用方是 zip 解包（仅 Windows）；非 Windows 只在测试里用到。
#[cfg(any(windows, test))]
fn safe_relative(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

/// 便携包可能带一层 `pengj-templates/`，也可能直接摊在根上。
fn bundle_root(staging: &Path) -> PathBuf {
    let nested = staging.join(BUNDLE_DIR);
    if nested.is_dir() {
        nested
    } else {
        staging.to_path_buf()
    }
}

// ---------------------------------------------------------------------------
// 通道二：本机构建
// ---------------------------------------------------------------------------

fn from_source(layout: &Layout, options: &Options) -> anyhow::Result<()> {
    let workspace = match &options.workspace {
        Some(dir) => dir.clone(),
        None => find_workspace().ok_or_else(|| {
            anyhow!("没找到工作区根（含 crates/cli 的 Cargo.toml），请在仓库内运行或用 --workspace 指定")
        })?,
    };
    let templates = workspace.join("templates");
    if !templates.is_dir() {
        bail!(
            "{} 不存在：--from-source 需要工作区自带模板树",
            templates.display()
        );
    }

    println!("工作区: {}", workspace.display());
    println!("编译: cargo build --release -p pengj-templates-cli");
    if options.dry_run {
        println!("[dry-run] 将安装构建产物到 {}", layout.exe.display());
        return Ok(());
    }

    let built = build(&workspace)?;
    println!("构建产物: {}", built.display());
    apply(layout, &built, &templates, options)?;

    // 本机构建的版本号来自 Cargo.toml，可能和已发布版本相同而内容不同，
    // 所以这里以实际装上的为准，不拿 current 作比较。
    match probe_version(&layout.exe) {
        Some(version) => {
            println!("验证: 已安装 {version}（本机构建）");
            update::record_installed(&version);
        }
        None => eprintln!("警告: 装好后无法读出 --version，请手工确认"),
    }
    closing_hint();
    Ok(())
}

fn build(workspace: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("cargo")
        .args([
            "build",
            "--release",
            "-p",
            "pengj-templates-cli",
            "--message-format=json-render-diagnostics",
        ])
        .current_dir(workspace)
        .stdout(Stdio::piped())
        // 编译进度照常打到终端，别让人对着黑屏等两分钟
        .stderr(Stdio::inherit())
        .output()
        .context("无法启动 cargo（未安装或不在 PATH）")?;

    if !output.status.success() {
        bail!(
            "cargo build 失败（退出码 {:?}），先修掉编译错误再升级",
            output.status.code()
        );
    }
    if let Some(path) = artifact_from_messages(&output.stdout) {
        return Ok(path);
    }
    // 兜底：cargo 的 JSON 格式变过，拿不到就按默认布局猜
    let fallback = workspace.join("target").join("release").join(EXE_NAME);
    if fallback.is_file() {
        return Ok(fallback);
    }
    bail!("找不到构建产物：cargo 输出里没有可执行文件路径");
}

/// 从 `cargo build --message-format=json` 的流里捞出 bin 的绝对路径。
///
/// 比猜 `target/release` 稳：`CARGO_TARGET_DIR`、`--target` 一改，猜的路径就落空。
fn artifact_from_messages(stdout: &[u8]) -> Option<PathBuf> {
    let text = String::from_utf8_lossy(stdout);
    let mut found = None;
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(|reason| reason.as_str()) != Some("compiler-artifact") {
            continue;
        }
        if value["target"]["name"].as_str() != Some("pengj-templates-cli") {
            continue;
        }
        if let Some(path) = value.get("executable").and_then(|path| path.as_str()) {
            found = Some(PathBuf::from(path));
        }
    }
    found
}

fn find_workspace() -> Option<PathBuf> {
    let mut starts = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        starts.push(cwd);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            starts.push(dir.to_path_buf());
        }
    }
    for start in starts {
        for dir in start.ancestors() {
            if dir.join("crates").join("cli").join("Cargo.toml").is_file() {
                return Some(dir.to_path_buf());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 落盘
// ---------------------------------------------------------------------------

fn apply(
    layout: &Layout,
    source_exe: &Path,
    source_templates: &Path,
    options: &Options,
) -> anyhow::Result<()> {
    install_binary(source_exe, &layout.exe)?;
    println!("安装 CLI: {}", layout.exe.display());
    install_templates(source_templates, &layout.templates)?;
    println!("安装模板: {}", layout.templates.display());
    if options.no_path_refresh {
        println!("跳过 PATH 副本刷新（--no-path-refresh）");
    } else {
        refresh_path_copies(layout);
    }
    Ok(())
}

/// 就地替换一个可执行文件；目标正在运行也能换。
fn install_binary(source: &Path, dest: &Path) -> anyhow::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("无法创建目录 {}", parent.display()))?;
    }
    let name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(EXE_NAME);
    // 上一轮升级留下的 `.old-<pid>` 这时多半已经解除占用，顺手清掉；
    // 名字带 pid 就不会互相覆盖，所以必须按前缀扫，不能只删自己这一轮的。
    if let Some(parent) = dest.parent() {
        sweep(parent, &format!("{name}.old-"));
    }
    if !dest.exists() {
        fs::copy(source, dest).with_context(|| format!("写入 {} 失败", dest.display()))?;
        return Ok(());
    }
    // 目标可能正是当前进程的镜像：Windows 不允许覆盖运行中的 exe，但允许改名。
    let backup = backup_path(dest, "old");
    remove_file_if_exists(&backup);
    fs::rename(dest, &backup)
        .with_context(|| format!("无法替换 {}（文件被占用？）", dest.display()))?;
    if let Err(error) = fs::copy(source, dest) {
        let _ = fs::rename(&backup, dest);
        return Err(error).with_context(|| format!("写入 {} 失败，已回滚", dest.display()));
    }
    // 旧镜像可能还映射在别的进程里，删不掉就先留着，下次升级再清
    let _ = fs::remove_file(&backup);
    Ok(())
}

/// 整目录替换：先在同一父目录里铺好新的，再换名，避免「删了一半」的中间态。
fn install_templates(source: &Path, dest: &Path) -> anyhow::Result<()> {
    let parent = dest
        .parent()
        .ok_or_else(|| anyhow!("模板目录没有父目录: {}", dest.display()))?;
    fs::create_dir_all(parent)?;
    // 上次中途崩掉留下的半成品，先清掉
    sweep(parent, ".templates-incoming-");
    sweep(parent, ".templates-old-");

    let incoming = parent.join(format!(".templates-incoming-{}", std::process::id()));
    remove_dir_if_exists(&incoming);
    copy_dir(source, &incoming)
        .with_context(|| format!("复制模板到 {} 失败", incoming.display()))?;

    let backup = parent.join(format!(".templates-old-{}", std::process::id()));
    if dest.exists() {
        remove_dir_if_exists(&backup);
        fs::rename(dest, &backup)
            .with_context(|| format!("无法移走旧模板目录 {}", dest.display()))?;
    }
    if let Err(error) = fs::rename(&incoming, dest) {
        if backup.exists() {
            let _ = fs::rename(&backup, dest);
        }
        return Err(error).with_context(|| "启用新模板目录失败，已回滚".to_string());
    }
    remove_dir_if_exists(&backup);
    Ok(())
}

fn copy_dir(source: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&from, &to)?;
        } else if file_type.is_symlink() {
            // 模板树里没有软链，遇到就跳过，别把安装目录写成指向别处的东西
            continue;
        } else {
            fs::copy(&from, &to).with_context(|| format!("复制 {} 失败", from.display()))?;
        }
    }
    Ok(())
}

/// 把 PATH 上的其它副本一起换成新版本。
///
/// 不刷新的话，排在前面的旧副本会一直抢占命令名 —— 升级像是没生效。
fn refresh_path_copies(layout: &Layout) {
    let mut refreshed = 0usize;
    let mut failed = 0usize;
    for copy in path_copies() {
        if same_file(&copy, &layout.exe) {
            continue;
        }
        match install_binary(&layout.exe, &copy) {
            Ok(()) => {
                println!("刷新副本: {}", copy.display());
                refreshed += 1;
            }
            Err(error) => {
                eprintln!("警告: 副本 {} 刷新失败: {error}", copy.display());
                failed += 1;
            }
        }
    }
    if refreshed == 0 && failed == 0 {
        println!("PATH 上没有其它副本需要刷新");
    }
}

fn path_copies() -> Vec<PathBuf> {
    let Some(path) = env::var_os("PATH") else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = Vec::new();
    for dir in env::split_paths(&path) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(EXE_NAME);
        let Ok(meta) = fs::symlink_metadata(&candidate) else {
            continue;
        };
        // Unix 一键装常是软链，指向的就是安装根那一份；换成实体文件会把链路弄断
        if meta.file_type().is_symlink() || !meta.is_file() {
            continue;
        }
        // `target/debug`、`target/release` 是编译产物，不是安装副本：覆盖它会让
        // cargo 的增量判断与实际内容对不上，下一次构建还以为是最新的
        if is_build_artifact(&candidate) {
            continue;
        }
        if out.iter().any(|known| same_file(known, &candidate)) {
            continue;
        }
        out.push(candidate);
    }
    out
}

/// 路径是否落在 `<...>/target/{debug,release}/` 里（含 `target/<triple>/release/`）。
fn is_build_artifact(path: &Path) -> bool {
    let mut current = path.parent();
    let mut saw_profile = false;
    for _ in 0..3 {
        let Some(dir) = current else {
            return false;
        };
        match dir.file_name().and_then(|name| name.to_str()) {
            Some("debug") | Some("release") => saw_profile = true,
            Some("target") => return saw_profile,
            _ => {}
        }
        current = dir.parent();
    }
    false
}

fn same_file(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => left == right,
    }
}

// ---------------------------------------------------------------------------
// 收尾
// ---------------------------------------------------------------------------

fn verify(layout: &Layout, expected: Option<&Version>) -> anyhow::Result<()> {
    let probed = probe_version(&layout.exe);
    match (&probed, expected) {
        (Some(version), Some(expected)) => {
            println!("验证: 已安装 {version}");
            if version != expected {
                eprintln!(
                    "警告: 装上的版本是 {version}，目标是 {expected}；检查 PATH 上的旧副本是否还在抢占"
                );
            }
        }
        (Some(version), None) => println!("验证: 已安装 {version}"),
        (None, _) => eprintln!("警告: 无法从 {} 读出 --version", layout.exe.display()),
    }
    Ok(())
}

fn probe_version(exe: &Path) -> Option<Version> {
    let output = Command::new(exe).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Version::parse(text.split_whitespace().last()?)
}

fn closing_hint() {
    println!(
        "提示: 工具升级不会改动已纳管项目的受管产物，需要跟进时在项目里跑 `pengj-templates-cli update --dir .`"
    );
}

fn backup_path(dest: &Path, tag: &str) -> PathBuf {
    let name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(EXE_NAME);
    dest.with_file_name(format!("{name}.{tag}-{}", std::process::id()))
}

fn remove_file_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn remove_dir_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}

/// 清掉目录里以 `prefix` 开头的历史残留（上一次升级的备份、半途中断的临时目录）。
///
/// 删不掉不算错：正在运行的镜像一定删不掉，那是预期内的，下一轮再来。
fn sweep(dir: &Path, prefix: &str) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(prefix) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            let _ = fs::remove_dir_all(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
}

/// 用完即删的临时目录；进程被 Ctrl-C 掉也不会留下几十 MB 的残骸。
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> anyhow::Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0);
        let path = env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path)
            .with_context(|| format!("无法创建临时目录 {}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(raw: &str) -> Version {
        Version::parse(raw).unwrap_or_else(|| panic!("无法解析版本号 {raw}"))
    }

    #[test]
    fn 布局固定在安装根的_bin_与_templates() {
        let layout = Layout::at(PathBuf::from("/tmp/pengj"));
        assert_eq!(layout.exe, Path::new("/tmp/pengj/bin").join(EXE_NAME));
        assert_eq!(layout.templates, Path::new("/tmp/pengj/templates"));
    }

    #[test]
    fn 便携包名带系统与架构且扩展名按平台() {
        let name = bundle_asset_name("linux", "arm64");
        assert!(name.starts_with("pengj-templates-cli-linux-arm64."));
        assert!(name.ends_with(if cfg!(windows) { ".zip" } else { ".tar.gz" }));
    }

    #[test]
    fn 拒绝解包出界的路径() {
        assert_eq!(
            safe_relative(Path::new("pengj-templates/a.txt")),
            Some(PathBuf::from("pengj-templates/a.txt"))
        );
        assert_eq!(
            safe_relative(Path::new("./a.txt")),
            Some(PathBuf::from("a.txt"))
        );
        assert_eq!(safe_relative(Path::new("../evil.txt")), None);
        assert_eq!(safe_relative(Path::new("/etc/passwd")), None);
        assert_eq!(safe_relative(Path::new("")), None);
    }

    #[test]
    fn 便携包根优先取顶层目录() {
        let temp = TempDir::new("pengj-test-bundle").expect("临时目录");
        assert_eq!(bundle_root(temp.path()), temp.path());
        fs::create_dir_all(temp.path().join(BUNDLE_DIR)).expect("建目录");
        assert_eq!(bundle_root(temp.path()), temp.path().join(BUNDLE_DIR));
    }

    #[test]
    fn 从_cargo_json_里认出构建产物() {
        let output = br#"{"reason":"compiler-artifact","target":{"name":"pengj-templates-cli"},"executable":"/w/target/release/pengj-templates-cli"}
{"reason":"compiler-artifact","target":{"name":"pengj-core"},"executable":null}
{"reason":"build-finished","success":true}"#;
        assert_eq!(
            artifact_from_messages(output),
            Some(PathBuf::from("/w/target/release/pengj-templates-cli"))
        );
    }

    #[test]
    fn 没有产物时返回_none() {
        assert_eq!(artifact_from_messages(b"not json"), None);
    }

    #[test]
    fn 编译产物不算安装副本() {
        assert!(is_build_artifact(Path::new(
            "/w/target/debug/pengj-templates-cli"
        )));
        assert!(is_build_artifact(Path::new(
            "D:/w/target/release/pengj-templates-cli.exe"
        )));
        assert!(!is_build_artifact(Path::new(
            "C:/Users/me/.cargo/bin/pengj-templates-cli.exe"
        )));
        // 带目标三元组目录的构建产物同样跳过
        assert!(is_build_artifact(Path::new(
            "/w/target/x86_64-pc-windows-msvc/release/x.exe"
        )));
    }

    #[test]
    fn 备份名与目标同目录且带进程号() {
        let backup = backup_path(Path::new("/tmp/bin/pengj-templates-cli.exe"), "old");
        assert_eq!(backup.parent(), Some(Path::new("/tmp/bin")));
        assert!(backup
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("pengj-templates-cli.exe.old-")));
    }

    #[test]
    fn 降级与同版本默认不动() {
        let current = version("0.80.1");
        let older = version("0.79.0");
        assert!(older < current);
        assert!(version("0.81.0") > current);
    }
}
