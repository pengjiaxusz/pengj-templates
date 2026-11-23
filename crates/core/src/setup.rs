//! 跨平台一键装机（setup）：以 `.pengj-templates/manifest.json` 为唯一事实源，
//! 按项目启用的层推导装机计划，并通过单一高层「环境探针与执行器」接缝（[`SetupEnv`]）
//! 隔离操作系统探测、磁盘空间查询、免密 `sudo` 探测与外部命令执行，支持离线单元测试与
//! `--dry-run` 预演。

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use serde::Serialize;

use crate::doctor::{absolute, display_path, has_layer, resolve_lang, uses_lld};
use crate::error::Result;
use crate::i18n_audit::Lang;
use crate::manifest::ProjectManifest;
use crate::setup_ephemeral::{detect_ephemeral, execute_ephemeral_step, plan_ephemeral_step};

/// 临时受限环境自动判定的可用磁盘空间阈值（20 GiB）
pub const EPHEMERAL_AVAIL_THRESHOLD_BYTES: u64 = 20 * 1024 * 1024 * 1024;

/// 临时受限环境自动判定的分区总容量阈值（40 GiB）
pub const EPHEMERAL_TOTAL_THRESHOLD_BYTES: u64 = 40 * 1024 * 1024 * 1024;

/// 目标操作系统类型（供装机计划按平台裁剪，并允许测试注入任意平台）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SetupOs {
    Linux,
    MacOs,
    Windows,
}

impl SetupOs {
    /// 获取当前编译目标的操作系统
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(target_os = "macos")]
        {
            Self::MacOs
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            Self::Linux
        }
    }

    /// 平台对应的自定义扩展脚本后缀
    pub fn script_ext(self) -> &'static str {
        match self {
            Self::Windows => "ps1",
            Self::Linux | Self::MacOs => "sh",
        }
    }

    /// 平台对应的自定义扩展脚本通配符
    pub fn script_glob(self) -> &'static str {
        match self {
            Self::Windows => "*.ps1",
            Self::Linux | Self::MacOs => "*.sh",
        }
    }
}

/// 磁盘空间统计（字节）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DiskSpace {
    /// 可用字节数
    pub available_bytes: u64,
    /// 分区总字节数
    pub total_bytes: u64,
}

/// 外部命令执行规格
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    /// 可执行程序名或路径
    pub program: String,
    /// 命令行参数
    pub args: Vec<String>,
    /// 可选工作目录
    pub cwd: Option<PathBuf>,
    /// 需要设置的环境变量
    pub env: Vec<(String, String)>,
    /// 需要移除的环境变量
    pub env_remove: Vec<String>,
}

impl CommandSpec {
    /// 构造一个基础外部命令规格
    pub fn new(program: impl Into<String>, args: &[&str]) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            cwd: None,
            env: Vec::new(),
            env_remove: Vec::new(),
        }
    }

    /// 设置工作目录
    pub fn cwd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cwd = Some(dir.into());
        self
    }
}

/// 外部命令执行结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// 退出码（无法获取时为 -1）
    pub status_code: i32,
    /// 是否以 0 退出
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
}

impl CommandOutput {
    /// 构造成功输出
    pub fn ok(stdout: impl Into<String>) -> Self {
        Self {
            status_code: 0,
            success: true,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    /// 构造失败输出
    pub fn fail(status_code: i32, stderr: impl Into<String>) -> Self {
        Self {
            status_code,
            success: false,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }
}

/// 装机模块唯一的「环境探针与执行器」接缝。
///
/// 覆盖目标平台、命令存在性/版本探测、磁盘空间查询、环境变量读取、免密 `sudo -n` 探测、
/// 文件读写与外部命令执行；生产代码使用 [`SystemSetupEnv`]，离线测试使用 [`FakeSetupEnv`]。
pub trait SetupEnv {
    /// 当前目标操作系统
    fn target_os(&self) -> SetupOs;

    /// 探测指定命令是否可用，并返回首行版本字符串
    fn probe_command_version(&self, program: &str, args: &[&str]) -> Option<String>;

    /// 探测指定命令是否存在于 `PATH` 中
    fn has_command(&self, program: &str) -> bool {
        self.probe_command_version(program, &["--version"])
            .is_some()
    }

    /// 查询指定路径所在分区的磁盘空间
    fn disk_space(&self, path: &Path) -> Option<DiskSpace>;

    /// 读取环境变量
    fn env_var(&self, key: &str) -> Option<String>;

    /// 获取用户主目录
    fn home_dir(&self) -> Option<PathBuf>;

    /// 探测当前环境是否具备免密 `sudo -n` 权限
    fn has_passwordless_sudo(&self) -> bool;

    /// 路径是否存在
    fn path_exists(&self, path: &Path) -> bool;

    /// 路径是否为目录
    fn is_dir(&self, path: &Path) -> bool;

    /// 读取文本文件
    fn read_file(&self, path: &Path) -> std::io::Result<String>;

    /// 写入文本文件（自动补建父目录）
    fn write_file(&self, path: &Path, content: &str) -> std::io::Result<()>;

    /// 列出目录下的直接子文件名（不含子目录）
    fn list_dir_files(&self, dir: &Path) -> std::io::Result<Vec<String>>;

    /// 执行外部命令
    fn run_command(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput>;
}

/// 基于真实操作系统的 [`SetupEnv`] 默认实现
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemSetupEnv;

impl SetupEnv for SystemSetupEnv {
    fn target_os(&self) -> SetupOs {
        SetupOs::current()
    }

    fn probe_command_version(&self, program: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(program).args(args).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next().unwrap_or("").trim();
        if !line.is_empty() {
            return Some(line.to_string());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let err_line = stderr.lines().next().unwrap_or("").trim();
        Some(err_line.to_string())
    }

    fn disk_space(&self, path: &Path) -> Option<DiskSpace> {
        probe_system_disk_space(path)
    }

    fn env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var_os("USERPROFILE").map(PathBuf::from)
        }
        #[cfg(not(windows))]
        {
            std::env::var_os("HOME").map(PathBuf::from)
        }
    }

    fn has_passwordless_sudo(&self) -> bool {
        #[cfg(unix)]
        {
            Command::new("sudo")
                .args(["-n", "true"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            false
        }
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn read_file(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn write_file(&self, path: &Path, content: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    fn list_dir_files(&self, dir: &Path) -> std::io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    fn run_command(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput> {
        let mut cmd = Command::new(&spec.program);
        cmd.args(&spec.args);
        if let Some(cwd) = &spec.cwd {
            cmd.current_dir(cwd);
        }
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }
        for k in &spec.env_remove {
            cmd.env_remove(k);
        }
        let out = cmd.output()?;
        Ok(CommandOutput {
            status_code: out.status.code().unwrap_or(-1),
            success: out.status.success(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}

fn probe_system_disk_space(path: &Path) -> Option<DiskSpace> {
    #[cfg(unix)]
    {
        let output = Command::new("df").arg("-Pk").arg(path).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_posix_df_pk(&stdout)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

/// 解析 `df -Pk <path>` 输出（1024 字节块），返回总容量与可用空间
#[cfg(any(unix, test))]
fn parse_posix_df_pk(stdout: &str) -> Option<DiskSpace> {
    let data_line = stdout.lines().nth(1)?;
    let cols: Vec<&str> = data_line.split_whitespace().collect();
    if cols.len() < 4 {
        return None;
    }
    // POSIX `df -Pk` 列序：Filesystem 1024-blocks Used Available Capacity Mounted_on
    let total_kib: u64 = cols[1].parse().ok()?;
    let avail_kib: u64 = cols[3].parse().ok()?;
    Some(DiskSpace {
        available_bytes: avail_kib.saturating_mul(1024),
        total_bytes: total_kib.saturating_mul(1024),
    })
}

/// 离线测试用内存替身（可注入任意操作系统、已装命令、磁盘余量、环境变量与虚拟文件）
#[derive(Debug)]
pub struct FakeSetupEnv {
    os: SetupOs,
    commands: BTreeMap<String, String>,
    disk: Option<DiskSpace>,
    env_vars: BTreeMap<String, String>,
    home: Option<PathBuf>,
    passwordless_sudo: bool,
    files: Mutex<BTreeMap<PathBuf, String>>,
    dirs: Mutex<BTreeSet<PathBuf>>,
    command_outputs: Mutex<VecDeque<CommandOutput>>,
    executed: Mutex<Vec<CommandSpec>>,
}

impl FakeSetupEnv {
    /// 构造指定操作系统的内存替身（默认无已装工具、无 CI 环境变量、磁盘空间充足）
    pub fn new(os: SetupOs) -> Self {
        let default_home = match os {
            SetupOs::Windows => PathBuf::from(r"C:\Users\demo"),
            SetupOs::Linux | SetupOs::MacOs => PathBuf::from("/home/demo"),
        };
        Self {
            os,
            commands: BTreeMap::new(),
            disk: Some(DiskSpace {
                available_bytes: 100 * 1024 * 1024 * 1024,
                total_bytes: 256 * 1024 * 1024 * 1024,
            }),
            env_vars: BTreeMap::new(),
            home: Some(default_home),
            passwordless_sudo: true,
            files: Mutex::new(BTreeMap::new()),
            dirs: Mutex::new(BTreeSet::new()),
            command_outputs: Mutex::new(VecDeque::new()),
            executed: Mutex::new(Vec::new()),
        }
    }

    /// 注入一个已安装命令及其版本输出
    pub fn with_command(mut self, program: impl Into<String>, version: impl Into<String>) -> Self {
        self.commands.insert(program.into(), version.into());
        self
    }

    /// 设置磁盘空间查询结果
    pub fn with_disk_space(mut self, disk: Option<DiskSpace>) -> Self {
        self.disk = disk;
        self
    }

    /// 注入一个环境变量
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// 设置免密 `sudo -n` 探测结果
    pub fn with_passwordless_sudo(mut self, allowed: bool) -> Self {
        self.passwordless_sudo = allowed;
        self
    }

    /// 注入一个虚拟文件（同时登记其父目录）
    pub fn with_file(self, path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        let p = path.into();
        if let Some(parent) = p.parent() {
            if let Ok(mut dirs) = self.dirs.lock() {
                dirs.insert(parent.to_path_buf());
            }
        }
        if let Ok(mut files) = self.files.lock() {
            files.insert(p, content.into());
        }
        self
    }

    /// 预置一条外部命令返回结果（按 FIFO 顺序消费）
    pub fn push_command_output(&self, output: CommandOutput) {
        if let Ok(mut queue) = self.command_outputs.lock() {
            queue.push_back(output);
        }
    }

    /// 读取已记录的外部命令调用列表
    pub fn executed_commands(&self) -> Vec<CommandSpec> {
        self.executed.lock().map(|v| v.clone()).unwrap_or_default()
    }

    /// 设置模拟的用户主目录（传 `None` 模拟主目录不可用）
    pub fn with_home_dir(mut self, home: Option<PathBuf>) -> Self {
        self.home = home;
        self
    }

    /// 读取虚拟文件内容
    pub fn read_virtual_file(&self, path: &Path) -> Option<String> {
        self.files.lock().ok()?.get(path).cloned()
    }

    /// 列出所有已写入或预置的虚拟文件路径
    pub fn virtual_file_paths(&self) -> Vec<PathBuf> {
        self.files
            .lock()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl SetupEnv for FakeSetupEnv {
    fn target_os(&self) -> SetupOs {
        self.os
    }

    fn probe_command_version(&self, program: &str, _args: &[&str]) -> Option<String> {
        self.commands.get(program).cloned()
    }

    fn disk_space(&self, _path: &Path) -> Option<DiskSpace> {
        self.disk
    }

    fn env_var(&self, key: &str) -> Option<String> {
        self.env_vars.get(key).cloned()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.home.clone()
    }

    fn has_passwordless_sudo(&self) -> bool {
        self.passwordless_sudo
    }

    fn path_exists(&self, path: &Path) -> bool {
        if let Ok(files) = self.files.lock() {
            if files.contains_key(path) {
                return true;
            }
        }
        if let Ok(dirs) = self.dirs.lock() {
            if dirs.contains(path) {
                return true;
            }
        }
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        if let Ok(dirs) = self.dirs.lock() {
            if dirs.contains(path) {
                return true;
            }
        }
        path.is_dir()
    }

    fn read_file(&self, path: &Path) -> std::io::Result<String> {
        if let Ok(files) = self.files.lock() {
            if let Some(content) = files.get(path) {
                return Ok(content.clone());
            }
        }
        std::fs::read_to_string(path)
    }

    fn write_file(&self, path: &Path, content: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            if let Ok(mut dirs) = self.dirs.lock() {
                dirs.insert(parent.to_path_buf());
            }
        }
        if let Ok(mut files) = self.files.lock() {
            files.insert(path.to_path_buf(), content.to_string());
        }
        Ok(())
    }

    fn list_dir_files(&self, dir: &Path) -> std::io::Result<Vec<String>> {
        let mut names = BTreeSet::new();
        let mut found_virtual = false;
        if let Ok(files) = self.files.lock() {
            for p in files.keys() {
                if p.parent() == Some(dir) {
                    if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                        names.insert(name.to_string());
                        found_virtual = true;
                    }
                }
            }
        }
        if !found_virtual && dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        names.insert(name.to_string());
                    }
                }
            }
        }
        Ok(names.into_iter().collect())
    }

    fn run_command(&self, spec: &CommandSpec) -> std::io::Result<CommandOutput> {
        if let Ok(mut executed) = self.executed.lock() {
            executed.push(spec.clone());
        }
        if let Ok(mut queue) = self.command_outputs.lock() {
            if let Some(out) = queue.pop_front() {
                return Ok(out);
            }
        }
        Ok(CommandOutput::ok(""))
    }
}

/// 装机运行选项
#[derive(Debug, Clone, Default)]
pub struct SetupOptions {
    /// 仅计算并输出装机计划，不执行任何下载、安装或写入
    pub dry_run: bool,
    /// 显式覆盖临时受限环境判定：`Some(true)` 对应 `--ephemeral`/`--ci`，
    /// `Some(false)` 对应 `--no-ephemeral`，`None` 按 CI 与磁盘空间自动探测
    pub ephemeral: Option<bool>,
    /// 可选的输出语言覆盖（优先级高于 `manifest.options.skill_lang`）
    pub lang: Option<Lang>,
}

/// 单步装机状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SetupStatus {
    /// 预演计划中待执行
    Planned,
    /// 已完成安装或写入
    Installed,
    /// 已满足或当前项目不需要，幂等跳过
    Skipped,
    /// 执行失败（阻断，退出码为 1）
    Failed,
    /// 非阻断告警（退出码仍为 0）
    Warn,
}

/// 装机报告中的单个步骤
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SetupStep {
    /// 稳定步骤标识（如 `submodules`、`just`、`pwsh`、`bque`、`rust`、`lld`、`ephemeral`、`doctor-fix`、`setup-d`）
    pub id: String,
    /// 分类（如 `repo`、`tool`、`env`、`doctor`、`custom`）
    pub category: String,
    /// 步骤状态
    pub status: SetupStatus,
    /// 单行摘要说明
    pub summary: String,
    /// 可选详细信息（例如默认安装路径、体积参考、将写入的配置项等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// 可选修复或手动操作建议
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
}

/// 一次装机（或 `--dry-run` 预演）的完整结构化报告
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SetupReport {
    /// 项目根目录绝对路径
    pub root: String,
    /// 生效的输出语言（`zh` 或 `en`）
    pub lang: String,
    /// 是否为 `--dry-run` 预演模式
    pub dry_run: bool,
    /// manifest 中记录的已启用层列表
    pub layers: Vec<String>,
    /// 是否启用临时受限环境优化
    pub ephemeral: bool,
    /// 临时受限环境判定原因说明
    pub ephemeral_reason: String,
    /// 整体是否成功（无失败步骤即为 true）
    pub ok: bool,
    /// 总步骤数
    pub total: usize,
    /// 计划待执行步骤数
    pub planned: usize,
    /// 已安装/已写入步骤数
    pub installed: usize,
    /// 已跳过步骤数
    pub skipped: usize,
    /// 告警步骤数
    pub warnings: usize,
    /// 失败步骤数
    pub failed: usize,
    /// 逐项装机步骤
    pub steps: Vec<SetupStep>,
}

impl SetupReport {
    /// 统一退出码：存在失败步骤返回 `1`，否则返回 `0`（仅告警或预演仍返回 `0`）
    pub fn exit_code(&self) -> i32 {
        if self.failed > 0 {
            1
        } else {
            0
        }
    }
}

/// 使用真实系统环境执行装机（或 `--dry-run` 预演）。
///
/// 以 `.pengj-templates/manifest.json` 为唯一事实源；缺失时返回 `CoreError::MissingManifest`。
pub fn run_setup(project_dir: &Path, opts: &SetupOptions) -> Result<SetupReport> {
    let env = SystemSetupEnv;
    run_setup_with_env(project_dir, opts, &env)
}

/// 通过注入的 [`SetupEnv`] 接缝执行装机（或 `--dry-run` 预演），供核心逻辑与离线测试共用。
pub fn run_setup_with_env(
    project_dir: &Path,
    opts: &SetupOptions,
    env: &dyn SetupEnv,
) -> Result<SetupReport> {
    let root = absolute(project_dir)?;
    let manifest = ProjectManifest::load(&root)?;
    let lang = resolve_lang(&manifest, opts.lang);

    let (ephemeral, ephemeral_reason) = detect_ephemeral(&root, opts.ephemeral, env, lang);
    let planned_steps = build_setup_plan(&root, &manifest, env, ephemeral, &ephemeral_reason, lang);

    let steps = if opts.dry_run {
        planned_steps
    } else {
        planned_steps
            .into_iter()
            .map(|step| {
                execute_planned_step(&root, &manifest, &ephemeral_reason, env, opts, lang, step)
            })
            .collect()
    };

    let total = steps.len();
    let planned = steps
        .iter()
        .filter(|s| s.status == SetupStatus::Planned)
        .count();
    let installed = steps
        .iter()
        .filter(|s| s.status == SetupStatus::Installed)
        .count();
    let skipped = steps
        .iter()
        .filter(|s| s.status == SetupStatus::Skipped)
        .count();
    let warnings = steps
        .iter()
        .filter(|s| s.status == SetupStatus::Warn)
        .count();
    let failed = steps
        .iter()
        .filter(|s| s.status == SetupStatus::Failed)
        .count();

    Ok(SetupReport {
        root: display_path(&root),
        lang: lang.as_str().to_string(),
        dry_run: opts.dry_run,
        layers: manifest.layers,
        ephemeral,
        ephemeral_reason,
        ok: failed == 0,
        total,
        planned,
        installed,
        skipped,
        warnings,
        failed,
        steps,
    })
}

// ---------------------------------------------------------------------------
// 装机计划推导（--dry-run 与执行模式共享同一份推导）
// ---------------------------------------------------------------------------

fn build_setup_plan(
    root: &Path,
    manifest: &ProjectManifest,
    env: &dyn SetupEnv,
    ephemeral: bool,
    ephemeral_reason: &str,
    lang: Lang,
) -> Vec<SetupStep> {
    let os = env.target_os();
    let mut steps = Vec::new();

    // 1. 子模块拉齐
    if let Some(step) = plan_submodules(root, manifest, env, lang) {
        steps.push(step);
    }

    // 2. just 层（以及 Windows 下的 pwsh）
    if has_layer(manifest, "just") {
        steps.push(plan_just(os, env, lang));
        if os == SetupOs::Windows {
            steps.push(plan_pwsh(env, lang));
        }
    }

    // 3. bque 层
    if has_layer(manifest, "bque") {
        steps.push(plan_bque(os, env, lang));
    }

    // 4. rust / rust-workspace 层（含 Linux 且启用 use_lld 时的 lld）
    let has_rust = has_layer(manifest, "rust") || has_layer(manifest, "rust-workspace");
    if has_rust {
        steps.push(plan_rust(root, os, env, lang));
        if os == SetupOs::Linux && uses_lld(manifest) {
            steps.push(plan_lld(env, lang));
        }
    }

    // 5. 临时受限环境优化（见 setup_ephemeral.rs）
    steps.push(plan_ephemeral_step(
        has_rust,
        ephemeral,
        ephemeral_reason,
        os,
        env,
        lang,
    ));

    // 6. 本地离线初始化：doctor --fix（占位步骤，实际调用由下游工单 #170 填充）
    steps.push(plan_doctor_fix(lang));

    // 7. 项目自定义扩展：.pengj-templates/scripts/setup.d/（占位步骤，实际执行由下游工单 #171 填充）
    steps.push(plan_setup_d(root, os, env, lang));

    steps
}

fn plan_submodules(
    root: &Path,
    manifest: &ProjectManifest,
    env: &dyn SetupEnv,
    lang: Lang,
) -> Option<SetupStep> {
    let gitmodules_path = root.join(".gitmodules");
    let has_gitmodules = env
        .read_file(&gitmodules_path)
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    let has_subrepo = has_layer(manifest, "subrepo");

    if !has_gitmodules && !has_subrepo {
        return None;
    }

    if !has_gitmodules {
        return Some(SetupStep {
            id: "submodules".to_string(),
            category: "repo".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => "未检测到非空 .gitmodules，跳过子模块拉齐".to_string(),
                Lang::En => "no non-empty .gitmodules found; skipping submodule sync".to_string(),
            },
            detail: None,
            remedy: None,
        });
    }

    Some(SetupStep {
        id: "submodules".to_string(),
        category: "repo".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => "递归拉齐未初始化的 git 子模块".to_string(),
            Lang::En => "recursively initialize uninitialized git submodules".to_string(),
        },
        detail: Some(match lang {
            Lang::Zh => {
                "命令：git submodule update --init --recursive（按仓库子模块体积拉取）".to_string()
            }
            Lang::En => {
                "command: git submodule update --init --recursive (size depends on submodules)"
                    .to_string()
            }
        }),
        remedy: None,
    })
}

fn plan_just(os: SetupOs, env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    if let Some(ver) = env.probe_command_version("just", &["--version"]) {
        let trimmed = ver.strip_prefix("just ").unwrap_or(&ver).trim();
        return SetupStep {
            id: "just".to_string(),
            category: "tool".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => format!("just 已安装（{trimmed}），跳过"),
                Lang::En => format!("just already installed ({trimmed}); skipped"),
            },
            detail: None,
            remedy: None,
        };
    }

    let target = match os {
        SetupOs::Windows => r"%LOCALAPPDATA%\just\bin\just.exe",
        SetupOs::Linux | SetupOs::MacOs => "~/.local/bin/just",
    };

    SetupStep {
        id: "just".to_string(),
        category: "tool".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => "将安装 just（任务运行器）".to_string(),
            Lang::En => "will install just (command runner)".to_string(),
        },
        detail: Some(match lang {
            Lang::Zh => format!("默认路径：{target}；大致体积：约 3–5 MB"),
            Lang::En => format!("default path: {target}; approx size: ~3-5 MB"),
        }),
        remedy: Some(match lang {
            Lang::Zh => "手动安装参考：https://github.com/casey/just#installation".to_string(),
            Lang::En => "manual install: https://github.com/casey/just#installation".to_string(),
        }),
    }
}

fn plan_pwsh(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    if let Some(ver) = env.probe_command_version("pwsh", &["--version"]) {
        return SetupStep {
            id: "pwsh".to_string(),
            category: "tool".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => format!("pwsh 已安装（{}），跳过", ver.trim()),
                Lang::En => format!("pwsh already installed ({}); skipped", ver.trim()),
            },
            detail: None,
            remedy: None,
        };
    }

    SetupStep {
        id: "pwsh".to_string(),
        category: "tool".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => "将安装 pwsh（PowerShell 7，Windows 上的 just 配方需要）".to_string(),
            Lang::En => {
                "will install pwsh (PowerShell 7, required by just recipes on Windows)".to_string()
            }
        },
        detail: Some(match lang {
            Lang::Zh => {
                r"默认路径：%ProgramFiles%\PowerShell\7\pwsh.exe（通过 winget）；大致体积：约 100 MB"
                    .to_string()
            }
            Lang::En => {
                r"default path: %ProgramFiles%\PowerShell\7\pwsh.exe (via winget); approx size: ~100 MB"
                    .to_string()
            }
        }),
        remedy: Some(match lang {
            Lang::Zh => "手动安装命令：winget install Microsoft.PowerShell".to_string(),
            Lang::En => "manual install: winget install Microsoft.PowerShell".to_string(),
        }),
    }
}

fn plan_bque(os: SetupOs, env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    if let Some(ver) = env.probe_command_version("bque", &["--version"]) {
        return SetupStep {
            id: "bque".to_string(),
            category: "tool".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => format!("bque 已安装（{}），跳过", ver.trim()),
                Lang::En => format!("bque already installed ({}); skipped", ver.trim()),
            },
            detail: None,
            remedy: None,
        };
    }

    let (target, manual_cmd) = match os {
        SetupOs::Windows => (
            r"%LOCALAPPDATA%\bque\bin\bque.exe",
            "irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex",
        ),
        SetupOs::Linux | SetupOs::MacOs => (
            "~/.local/bin/bque",
            "curl -fsSL https://github.com/pengjiaxusz/bque/releases/latest/download/install.sh | sh",
        ),
    };

    SetupStep {
        id: "bque".to_string(),
        category: "tool".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => "将安装 bque（构建队列工具）".to_string(),
            Lang::En => "will install bque (build queue runner)".to_string(),
        },
        detail: Some(match lang {
            Lang::Zh => format!("默认路径：{target}；大致体积：约 3–5 MB"),
            Lang::En => format!("default path: {target}; approx size: ~3-5 MB"),
        }),
        remedy: Some(match lang {
            Lang::Zh => format!("手动安装命令：{manual_cmd}"),
            Lang::En => format!("manual install: {manual_cmd}"),
        }),
    }
}

fn plan_rust(root: &Path, os: SetupOs, env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    let rustup = env.probe_command_version("rustup", &["--version"]);
    let cargo = env.probe_command_version("cargo", &["--version"]);
    let rustc = env.probe_command_version("rustc", &["--version"]);
    let has_toolchain_file = env.path_exists(&root.join("rust-toolchain.toml"));

    let target = match os {
        SetupOs::Windows => r"%USERPROFILE%\.cargo\bin / %USERPROFILE%\.rustup",
        SetupOs::Linux | SetupOs::MacOs => "~/.cargo/bin / ~/.rustup",
    };

    if rustup.is_some() && cargo.is_some() && rustc.is_some() && !has_toolchain_file {
        let rustc_ver = rustc.unwrap_or_default();
        return SetupStep {
            id: "rust".to_string(),
            category: "tool".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => format!("Rust 工具链已就绪（{rustc_ver}），跳过"),
                Lang::En => format!("Rust toolchain already installed ({rustc_ver}); skipped"),
            },
            detail: None,
            remedy: None,
        };
    }

    if rustup.is_some() && has_toolchain_file {
        return SetupStep {
            id: "rust".to_string(),
            category: "tool".to_string(),
            status: SetupStatus::Planned,
            summary: match lang {
                Lang::Zh => "将通过 rustup 按 rust-toolchain.toml 同步工具链与组件".to_string(),
                Lang::En => {
                    "will sync Rust toolchain and components from rust-toolchain.toml via rustup"
                        .to_string()
                }
            },
            detail: Some(match lang {
                Lang::Zh => {
                    format!("默认路径：{target}；大致体积：按缺口组件约 50–400 MB")
                }
                Lang::En => {
                    format!("default path: {target}; approx size: ~50-400 MB depending on missing components")
                }
            }),
            remedy: None,
        };
    }

    SetupStep {
        id: "rust".to_string(),
        category: "tool".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => {
                "将安装 rustup 与 Rust 工具链（cargo / rustc / rustfmt / clippy）".to_string()
            }
            Lang::En => "will install rustup and Rust toolchain (cargo / rustc / rustfmt / clippy)"
                .to_string(),
        },
        detail: Some(match lang {
            Lang::Zh => format!("默认路径：{target}；大致体积：约 400–800 MB"),
            Lang::En => format!("default path: {target}; approx size: ~400-800 MB"),
        }),
        remedy: Some(match lang {
            Lang::Zh => "手动安装参考：https://rustup.rs".to_string(),
            Lang::En => "manual install: https://rustup.rs".to_string(),
        }),
    }
}

fn plan_lld(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    if let Some(ver) = env.probe_command_version("ld.lld", &["--version"]) {
        return SetupStep {
            id: "lld".to_string(),
            category: "tool".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => format!("ld.lld 已安装（{}），跳过", ver.trim()),
                Lang::En => format!("ld.lld already installed ({}); skipped", ver.trim()),
            },
            detail: None,
            remedy: None,
        };
    }

    let has_sudo = env.has_passwordless_sudo();
    let detail = match (lang, has_sudo) {
        (Lang::Zh, true) => {
            "默认路径：/usr/bin/ld.lld；大致体积：约 20–50 MB（已检测到免密 sudo，将通过系统包管理器安装）"
                .to_string()
        }
        (Lang::Zh, false) => {
            "默认路径：/usr/bin/ld.lld；大致体积：约 20–50 MB（未检测到免密 sudo，执行时将报错并给出手动安装命令）"
                .to_string()
        }
        (Lang::En, true) => {
            "default path: /usr/bin/ld.lld; approx size: ~20-50 MB (passwordless sudo available; will install via system package manager)"
                .to_string()
        }
        (Lang::En, false) => {
            "default path: /usr/bin/ld.lld; approx size: ~20-50 MB (no passwordless sudo; execution will report manual install command)"
                .to_string()
        }
    };

    SetupStep {
        id: "lld".to_string(),
        category: "tool".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => "将安装 lld 链接器（ld.lld）".to_string(),
            Lang::En => "will install lld linker (ld.lld)".to_string(),
        },
        detail: Some(detail),
        remedy: Some(match lang {
            Lang::Zh => {
                "手动安装：sudo apt install lld（Debian/Ubuntu）、sudo dnf install lld（Fedora/RHEL）或 sudo pacman -S lld（Arch）"
                    .to_string()
            }
            Lang::En => {
                "manual install: sudo apt install lld (Debian/Ubuntu), sudo dnf install lld (Fedora/RHEL), or sudo pacman -S lld (Arch)"
                    .to_string()
            }
        }),
    }
}

fn plan_doctor_fix(lang: Lang) -> SetupStep {
    SetupStep {
        id: "doctor-fix".to_string(),
        category: "doctor".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => "执行离线体检修复（doctor --fix）：激活提交钩子、工作区链接与共享编译目录"
                .to_string(),
            Lang::En => {
                "run offline doctor --fix: activate commit hooks, workspace links, and shared build directory"
                    .to_string()
            }
        },
        detail: Some(match lang {
            Lang::Zh => "纯离线进程内调用，不发起任何外网请求".to_string(),
            Lang::En => "runs purely offline in-process without network requests".to_string(),
        }),
        remedy: None,
    }
}

fn plan_setup_d(root: &Path, os: SetupOs, env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    let dir = root.join(".pengj-templates/scripts/setup.d");
    let ext = os.script_ext();
    let glob = os.script_glob();

    let scripts: Vec<String> = env
        .list_dir_files(&dir)
        .unwrap_or_default()
        .into_iter()
        .filter(|name| {
            Path::new(name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case(ext))
                .unwrap_or(false)
        })
        .collect();

    if scripts.is_empty() {
        return SetupStep {
            id: "setup-d".to_string(),
            category: "custom".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => format!(
                    "未发现自定义装机扩展脚本（可在 .pengj-templates/scripts/setup.d/ 添加 {glob}）"
                ),
                Lang::En => format!(
                    "no custom setup extension scripts found (add {glob} under .pengj-templates/scripts/setup.d/)"
                ),
            },
            detail: None,
            remedy: None,
        };
    }

    let list = scripts.join(", ");
    SetupStep {
        id: "setup-d".to_string(),
        category: "custom".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => format!("将按字典序执行 {} 个自定义装机脚本", scripts.len()),
            Lang::En => format!(
                "will run {} custom setup extension script(s) in lexicographical order",
                scripts.len()
            ),
        },
        detail: Some(match lang {
            Lang::Zh => {
                format!("脚本列表：{list}（注入 SETUP_ROOT、SETUP_DRY_RUN、SETUP_EPHEMERAL）")
            }
            Lang::En => {
                format!("scripts: {list} (injects SETUP_ROOT, SETUP_DRY_RUN, SETUP_EPHEMERAL)")
            }
        }),
        remedy: None,
    }
}

// ---------------------------------------------------------------------------
// 真实执行骨架（下游工单 #169 / #170 / #171 按步骤 ID 逐项填充）
// ---------------------------------------------------------------------------

fn execute_planned_step(
    root: &Path,
    manifest: &ProjectManifest,
    ephemeral_reason: &str,
    env: &dyn SetupEnv,
    _opts: &SetupOptions,
    lang: Lang,
    step: SetupStep,
) -> SetupStep {
    if step.status != SetupStatus::Planned {
        return step;
    }

    match step.id.as_str() {
        "submodules" => crate::setup_tools::execute_submodules(root, env, lang),
        "just" => crate::setup_tools::execute_just(env, lang),
        "pwsh" => crate::setup_tools::execute_pwsh(env, lang),
        "bque" => crate::setup_tools::execute_bque(env, lang),
        "rust" => crate::setup_tools::execute_rust(root, env, lang),
        "lld" => crate::setup_tools::execute_lld(env, lang),
        "ephemeral" => execute_ephemeral_step(manifest, ephemeral_reason, env, lang, step),
        _ => {
            let skeleton_note = match lang {
                Lang::Zh => "装机骨架占位：实际执行逻辑待后续工单接入（当前未做修改）".to_string(),
                Lang::En => {
                    "setup skeleton placeholder: execution logic will be wired in follow-up tasks (no changes made)"
                        .to_string()
                }
            };

            let detail = match step.detail {
                Some(existing) => Some(format!("{existing}; {skeleton_note}")),
                None => Some(skeleton_note),
            };

            SetupStep {
                status: SetupStatus::Warn,
                detail,
                ..step
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 人类可读双语渲染
// ---------------------------------------------------------------------------

/// 将装机报告格式化为面向人的终端文本（遵循 `zh` / `en` 语言轴，英文模式零汉字）
pub fn format_setup_report(report: &SetupReport, lang: Lang) -> String {
    let mut out = String::new();

    match lang {
        Lang::Zh => {
            let mode = if report.dry_run {
                "装机计划（--dry-run 预演，不执行下载或写入）"
            } else {
                "装机执行"
            };
            let _ = writeln!(out, "[setup] {mode}：{}", report.root);
            let eph_state = if report.ephemeral { "开启" } else { "关闭" };
            let _ = writeln!(
                out,
                "[setup] 临时受限环境：{eph_state}（{}）",
                report.ephemeral_reason
            );
        }
        Lang::En => {
            let mode = if report.dry_run {
                "setup plan (--dry-run preview, no downloads or writes)"
            } else {
                "setup execution"
            };
            let _ = writeln!(out, "[setup] {mode}: {}", report.root);
            let eph_state = if report.ephemeral {
                "enabled"
            } else {
                "disabled"
            };
            let _ = writeln!(
                out,
                "[setup] ephemeral environment: {eph_state} ({})",
                report.ephemeral_reason
            );
        }
    }

    for step in &report.steps {
        let tag = match step.status {
            SetupStatus::Planned => "plan ",
            SetupStatus::Installed => "ok   ",
            SetupStatus::Skipped => "skip ",
            SetupStatus::Warn => "warn ",
            SetupStatus::Failed => "fail ",
        };
        let _ = writeln!(out, "[setup] {tag} [{}] {}", step.id, step.summary);
        if let Some(detail) = &step.detail {
            let _ = writeln!(out, "        {detail}");
        }
        if let Some(remedy) = &step.remedy {
            let _ = writeln!(out, "        -> {remedy}");
        }
    }

    match lang {
        Lang::Zh => {
            let _ = writeln!(
                out,
                "[setup] 共 {} 步：计划 {}，已安装 {}，跳过 {}，告警 {}，失败 {}",
                report.total,
                report.planned,
                report.installed,
                report.skipped,
                report.warnings,
                report.failed
            );
        }
        Lang::En => {
            let _ = writeln!(
                out,
                "[setup] {} steps: {} planned, {} installed, {} skipped, {} warnings, {} failed",
                report.total,
                report.planned,
                report.installed,
                report.skipped,
                report.warnings,
                report.failed
            );
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::i18n_audit::contains_han;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-setup-{}-{}-{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建临时目录失败");
        dir
    }

    fn write_manifest(dir: &Path, layers: &[&str], options: BTreeMap<String, serde_json::Value>) {
        let manifest = ProjectManifest {
            tool: "pengj-templates".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            project_name: "demo".to_string(),
            layers: layers.iter().map(|s| (*s).to_string()).collect(),
            options,
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            files: BTreeMap::new(),
        };
        manifest.save(dir).expect("写入 manifest 失败");
    }

    fn assert_json_keys_ascii(value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    assert!(!contains_han(k), "JSON 键名不应含汉字: {k}");
                    assert!(k.is_ascii(), "JSON 键名应为全英文 ASCII: {k}");
                    assert_json_keys_ascii(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    assert_json_keys_ascii(item);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn 缺失manifest时返回missing_manifest错误() {
        let dir = temp_dir("missing-manifest");
        let env = FakeSetupEnv::new(SetupOs::Linux);
        let err = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &env,
        )
        .expect_err("缺失 manifest 必须报错");
        assert!(
            matches!(err, CoreError::MissingManifest(_)),
            "预期 MissingManifest，实际为: {err:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 按层裁剪dry_run装机计划() {
        let dir = temp_dir("layer-pruning");
        // 仅 common 层：不应出现 just / pwsh / bque / rust / lld / submodules
        write_manifest(&dir, &["common"], BTreeMap::new());
        let env = FakeSetupEnv::new(SetupOs::Linux);
        let report_common = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &env,
        )
        .expect("common 层预演应成功");
        let ids: Vec<&str> = report_common.steps.iter().map(|s| s.id.as_str()).collect();
        assert!(!ids.contains(&"submodules"));
        assert!(!ids.contains(&"just"));
        assert!(!ids.contains(&"pwsh"));
        assert!(!ids.contains(&"bque"));
        assert!(!ids.contains(&"rust"));
        assert!(!ids.contains(&"lld"));
        assert!(ids.contains(&"ephemeral"));
        assert!(ids.contains(&"doctor-fix"));
        assert!(ids.contains(&"setup-d"));

        // Linux 启用 just + bque + rust + subrepo：应包含 just / bque / rust / lld / submodules，不含 pwsh
        write_manifest(
            &dir,
            &["common", "just", "bque", "rust", "subrepo"],
            BTreeMap::new(),
        );
        let report_linux = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &env,
        )
        .expect("Linux 全层预演应成功");
        let ids_linux: Vec<&str> = report_linux.steps.iter().map(|s| s.id.as_str()).collect();
        assert!(ids_linux.contains(&"submodules"));
        assert!(ids_linux.contains(&"just"));
        assert!(!ids_linux.contains(&"pwsh"));
        assert!(ids_linux.contains(&"bque"));
        assert!(ids_linux.contains(&"rust"));
        assert!(ids_linux.contains(&"lld"));

        // Windows 启用 just + rust：应包含 pwsh，不含 lld
        let env_win = FakeSetupEnv::new(SetupOs::Windows);
        let report_win = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &env_win,
        )
        .expect("Windows 预演应成功");
        let ids_win: Vec<&str> = report_win.steps.iter().map(|s| s.id.as_str()).collect();
        assert!(ids_win.contains(&"pwsh"));
        assert!(!ids_win.contains(&"lld"));

        // Linux 上关闭 use_lld=false：应裁剪 lld
        let mut opts = BTreeMap::new();
        opts.insert("use_lld".to_string(), serde_json::json!(false));
        write_manifest(&dir, &["common", "rust"], opts);
        let report_no_lld = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &env,
        )
        .expect("关闭 use_lld 预演应成功");
        assert!(!report_no_lld.steps.iter().any(|s| s.id == "lld"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 已安装工具在dry_run中标记为跳过且展示未装工具路径与体积() {
        let dir = temp_dir("installed-skip");
        write_manifest(&dir, &["common", "just", "bque", "rust"], BTreeMap::new());

        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_command("just", "just 1.40.0")
            .with_command("bque", "bque 0.5.0")
            .with_command("rustup", "rustup 1.28.1")
            .with_command("cargo", "cargo 1.85.0")
            .with_command("rustc", "rustc 1.85.0")
            .with_command("ld.lld", "Debian LLD 18.1.3");

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &env,
        )
        .expect("全已装预演应成功");

        for id in ["just", "bque", "rust", "lld"] {
            let step = report
                .steps
                .iter()
                .find(|s| s.id == id)
                .unwrap_or_else(|| panic!("应包含步骤 {id}"));
            assert_eq!(
                step.status,
                SetupStatus::Skipped,
                "已装工具 {id} 应标记为 Skipped"
            );
        }

        // 未安装工具时应标记 Planned 并带默认路径与大致体积参考
        let empty_env = FakeSetupEnv::new(SetupOs::Linux);
        let report_missing = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &empty_env,
        )
        .expect("未装工具预演应成功");

        let just_step = report_missing
            .steps
            .iter()
            .find(|s| s.id == "just")
            .expect("应包含 just");
        assert_eq!(just_step.status, SetupStatus::Planned);
        let just_detail = just_step.detail.as_deref().unwrap_or("");
        assert!(just_detail.contains("~/.local/bin/just"));
        assert!(just_detail.contains("MB"));

        let lld_step = report_missing
            .steps
            .iter()
            .find(|s| s.id == "lld")
            .expect("应包含 lld");
        assert_eq!(lld_step.status, SetupStatus::Planned);
        let lld_detail = lld_step.detail.as_deref().unwrap_or("");
        assert!(lld_detail.contains("/usr/bin/ld.lld"));
        assert!(lld_detail.contains("MB"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 临时受限环境判定矩阵与显式覆盖() {
        let dir = temp_dir("ephemeral-matrix");
        write_manifest(&dir, &["common", "rust"], BTreeMap::new());

        // 1. 磁盘充裕且无 CI -> false
        let ample = FakeSetupEnv::new(SetupOs::Linux).with_disk_space(Some(DiskSpace {
            available_bytes: 120 * 1024 * 1024 * 1024,
            total_bytes: 512 * 1024 * 1024 * 1024,
        }));
        let r1 = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &ample,
        )
        .expect("充裕磁盘预演应成功");
        assert!(!r1.ephemeral);

        // 2. 磁盘可用 < 20 GiB -> true
        let small_disk = FakeSetupEnv::new(SetupOs::Linux).with_disk_space(Some(DiskSpace {
            available_bytes: 12 * 1024 * 1024 * 1024,
            total_bytes: 64 * 1024 * 1024 * 1024,
        }));
        let r2 = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &small_disk,
        )
        .expect("小盘预演应成功");
        assert!(r2.ephemeral);
        let eph_step = r2
            .steps
            .iter()
            .find(|s| s.id == "ephemeral")
            .expect("应有 ephemeral 步骤");
        assert_eq!(eph_step.status, SetupStatus::Planned);

        // 3. CI=true -> true，但 --no-ephemeral 覆盖为 false
        let ci_env = FakeSetupEnv::new(SetupOs::Linux).with_env("CI", "true");
        let r3_ci = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &ci_env,
        )
        .expect("CI 预演应成功");
        assert!(r3_ci.ephemeral);

        let r3_override_off = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: Some(false),
                lang: None,
            },
            &ci_env,
        )
        .expect("--no-ephemeral 预演应成功");
        assert!(!r3_override_off.ephemeral);

        // 4. 磁盘充裕但显式 --ephemeral -> true
        let r4_override_on = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: Some(true),
                lang: None,
            },
            &ample,
        )
        .expect("--ephemeral 预演应成功");
        assert!(r4_override_on.ephemeral);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 中英双语零汉字混杂与json键名全英文() {
        let dir = temp_dir("bilingual-json");
        let mut opts = BTreeMap::new();
        opts.insert("skill_lang".to_string(), serde_json::json!("en"));
        write_manifest(&dir, &["common", "just", "bque", "rust", "subrepo"], opts);

        let abs = absolute(&dir).expect("路径归一化失败");
        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("CI", "true")
            .with_env("GITHUB_ENV", "/tmp/github_env")
            .with_passwordless_sudo(false)
            .with_file(abs.join(".gitmodules"), "[submodule \"a\"]\npath = a\n")
            .with_file(
                abs.join(".pengj-templates/scripts/setup.d/01-extra.sh"),
                "#!/bin/sh\n",
            );

        // 英文模式（来自 manifest skill_lang=en）：文本与全部字段零汉字，JSON 键名全英文
        let report_en = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: None,
            },
            &env,
        )
        .expect("英文预演应成功");
        assert_eq!(report_en.lang, "en");
        assert!(!contains_han(&report_en.ephemeral_reason));

        let rendered_en = format_setup_report(&report_en, Lang::En);
        assert!(
            !contains_han(&rendered_en),
            "英文模式渲染输出不应含汉字:\n{rendered_en}"
        );
        for step in &report_en.steps {
            assert!(
                !contains_han(&step.summary),
                "en summary 含汉字: {}",
                step.summary
            );
            if let Some(d) = &step.detail {
                assert!(!contains_han(d), "en detail 含汉字: {d}");
            }
            if let Some(r) = &step.remedy {
                assert!(!contains_han(r), "en remedy 含汉字: {r}");
            }
        }

        let json_en = serde_json::to_value(&report_en).expect("JSON 序列化应成功");
        assert_json_keys_ascii(&json_en);

        // 中文模式（--lang zh 覆盖）：包含「装机」「临时受限环境」术语，且 JSON 键名仍全英文
        let report_zh = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: true,
                ephemeral: None,
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("中文预演应成功");
        assert_eq!(report_zh.lang, "zh");
        let rendered_zh = format_setup_report(&report_zh, Lang::Zh);
        assert!(contains_han(&rendered_zh));
        assert!(rendered_zh.contains("装机"));
        assert!(rendered_zh.contains("临时受限环境"));

        let json_zh = serde_json::to_value(&report_zh).expect("中文报告 JSON 序列化应成功");
        assert_json_keys_ascii(&json_zh);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 真实执行骨架模式可运行且遵循统一退出码() {
        let dir = temp_dir("execute-skeleton");
        write_manifest(&dir, &["common", "just"], BTreeMap::new());

        let env = FakeSetupEnv::new(SetupOs::Linux).with_command("just", "just 1.40.0");
        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::En),
            },
            &env,
        )
        .expect("骨架执行应成功");

        assert!(!report.dry_run);
        assert_eq!(report.exit_code(), 0);
        let just_step = report
            .steps
            .iter()
            .find(|s| s.id == "just")
            .expect("应包含 just");
        assert_eq!(just_step.status, SetupStatus::Skipped);
        let doctor_step = report
            .steps
            .iter()
            .find(|s| s.id == "doctor-fix")
            .expect("应包含 doctor-fix");
        assert_eq!(doctor_step.status, SetupStatus::Warn);

        let rendered = format_setup_report(&report, Lang::En);
        assert!(
            !contains_han(&rendered),
            "英文执行模式输出不应含汉字:\n{rendered}"
        );

        let mut failed_report = report.clone();
        failed_report.failed = 1;
        failed_report.ok = false;
        assert_eq!(failed_report.exit_code(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn posix_df输出解析准确() {
        let sample = "\
Filesystem     1024-blocks      Used Available Capacity Mounted on
/dev/sda1         20971520  10485760  10485760      50% /
";
        let disk = parse_posix_df_pk(sample).expect("应成功解析 df -Pk");
        assert_eq!(disk.total_bytes, 20_971_520 * 1024);
        assert_eq!(disk.available_bytes, 10_485_760 * 1024);
    }
}
