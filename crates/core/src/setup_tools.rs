//! `setup` 子命令的真实工具安装与子模块拉齐执行实现。
//!
//! 所有外部命令、环境变量、免密 `sudo` 探测与文件读写均通过 [`SetupEnv`] 接缝执行，
//! 保证 100% 离线可测且全程零交互阻塞。

use std::path::{Path, PathBuf};

use crate::i18n_audit::Lang;
use crate::setup::{CommandOutput, CommandSpec, SetupEnv, SetupOs, SetupStatus, SetupStep};

/// `bque` 官方 Linux/macOS 安装脚本地址
const BQUE_INSTALL_SH_URL: &str =
    "https://github.com/pengjiaxusz/bque/releases/latest/download/install.sh";

/// `bque` 官方 Windows 安装脚本地址
const BQUE_INSTALL_PS1_URL: &str =
    "https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1";

/// `just` 官方 Unix 安装脚本地址
const JUST_INSTALL_SH_URL: &str = "https://just.systems/install.sh";

/// `rustup` 官方 Unix 安装脚本地址
const RUSTUP_INSTALL_SH_URL: &str = "https://sh.rustup.rs";

/// 当环境变量 `GITHUB_PATH` 存在时，幂等追加工具目录到 `GITHUB_PATH` 文件（不重复写入）。
///
/// 返回 `Ok(true)` 表示本次实际追加了新目录，`Ok(false)` 表示未设置 `GITHUB_PATH` 或该目录已存在。
pub(crate) fn append_github_path(env: &dyn SetupEnv, dir: &str) -> std::io::Result<bool> {
    let dir_trimmed = dir.trim();
    if dir_trimmed.is_empty() {
        return Ok(false);
    }
    let Some(gh_path_raw) = env.env_var("GITHUB_PATH") else {
        return Ok(false);
    };
    let gh_path_trimmed = gh_path_raw.trim();
    if gh_path_trimmed.is_empty() {
        return Ok(false);
    }

    let gh_path = PathBuf::from(gh_path_trimmed);
    let existing = match env.read_file(&gh_path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(_) if !env.path_exists(&gh_path) => String::new(),
        Err(err) => return Err(err),
    };

    if existing.lines().any(|line| line.trim() == dir_trimmed) {
        return Ok(false);
    }

    let updated = if existing.is_empty() {
        format!("{dir_trimmed}\n")
    } else if existing.ends_with('\n') {
        format!("{existing}{dir_trimmed}\n")
    } else {
        format!("{existing}\n{dir_trimmed}\n")
    };

    env.write_file(&gh_path, &updated)?;
    Ok(true)
}

/// 执行 `submodules` 步骤：检测 `.gitmodules` 存在且有未初始化子模块时递归初始化，否则跳过。
pub(crate) fn execute_submodules(root: &Path, env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    let has_gitmodules = env
        .read_file(&root.join(".gitmodules"))
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    if !has_gitmodules {
        return SetupStep {
            id: "submodules".to_string(),
            category: "repo".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => "未检测到非空 .gitmodules，跳过子模块拉齐".to_string(),
                Lang::En => "no non-empty .gitmodules found; skipping submodule sync".to_string(),
            },
            detail: None,
            remedy: None,
        };
    }

    let status_spec = CommandSpec::new("git", &["submodule", "status", "--recursive"]).cwd(root);
    let status_out = match env.run_command(&status_spec) {
        Ok(out) => out,
        Err(err) => {
            return failed_step(
                "submodules",
                "repo",
                match lang {
                    Lang::Zh => "检查 git 子模块状态失败".to_string(),
                    Lang::En => "failed to check git submodule status".to_string(),
                },
                format_io_error(lang, "git submodule status --recursive", &err),
                match lang {
                    Lang::Zh => "手动执行：git submodule update --init --recursive".to_string(),
                    Lang::En => {
                        "manual command: git submodule update --init --recursive".to_string()
                    }
                },
            );
        }
    };

    if !status_out.success {
        return failed_step(
            "submodules",
            "repo",
            match lang {
                Lang::Zh => "检查 git 子模块状态失败".to_string(),
                Lang::En => "failed to check git submodule status".to_string(),
            },
            format_cmd_failure(lang, "git submodule status --recursive", &status_out),
            match lang {
                Lang::Zh => "手动执行：git submodule update --init --recursive".to_string(),
                Lang::En => "manual command: git submodule update --init --recursive".to_string(),
            },
        );
    }

    let has_uninitialized = status_out.stdout.lines().any(|line| line.starts_with('-'));

    if !has_uninitialized {
        return SetupStep {
            id: "submodules".to_string(),
            category: "repo".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => "所有 git 子模块已初始化，跳过".to_string(),
                Lang::En => "all git submodules are already initialized; skipped".to_string(),
            },
            detail: None,
            remedy: None,
        };
    }

    let update_spec =
        CommandSpec::new("git", &["submodule", "update", "--init", "--recursive"]).cwd(root);
    match env.run_command(&update_spec) {
        Ok(out) if out.success => SetupStep {
            id: "submodules".to_string(),
            category: "repo".to_string(),
            status: SetupStatus::Installed,
            summary: match lang {
                Lang::Zh => "已递归初始化未就绪的 git 子模块".to_string(),
                Lang::En => "recursively initialized git submodules".to_string(),
            },
            detail: Some(match lang {
                Lang::Zh => "命令：git submodule update --init --recursive".to_string(),
                Lang::En => "command: git submodule update --init --recursive".to_string(),
            }),
            remedy: None,
        },
        Ok(out) => failed_step(
            "submodules",
            "repo",
            match lang {
                Lang::Zh => "递归初始化 git 子模块失败".to_string(),
                Lang::En => "failed to recursively initialize git submodules".to_string(),
            },
            format_cmd_failure(lang, "git submodule update --init --recursive", &out),
            match lang {
                Lang::Zh => "手动执行：git submodule update --init --recursive".to_string(),
                Lang::En => "manual command: git submodule update --init --recursive".to_string(),
            },
        ),
        Err(err) => failed_step(
            "submodules",
            "repo",
            match lang {
                Lang::Zh => "递归初始化 git 子模块失败".to_string(),
                Lang::En => "failed to recursively initialize git submodules".to_string(),
            },
            format_io_error(lang, "git submodule update --init --recursive", &err),
            match lang {
                Lang::Zh => "手动执行：git submodule update --init --recursive".to_string(),
                Lang::En => "manual command: git submodule update --init --recursive".to_string(),
            },
        ),
    }
}

/// 执行 `just` 步骤：Unix 优先免 `sudo` 安装到用户目录 `~/.local/bin`，Windows 走 `winget` 或明确指引。
pub(crate) fn execute_just(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
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

    match env.target_os() {
        SetupOs::Linux | SetupOs::MacOs => execute_just_unix(env, lang),
        SetupOs::Windows => execute_just_windows(env, lang),
    }
}

fn execute_just_unix(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    let bin_dir = unix_local_bin_dir(env);
    let quoted_dir = shell_single_quote(&bin_dir);
    let script = format!(
        "mkdir -p {quoted_dir} && curl --proto '=https' --tlsv1.2 -sSf {JUST_INSTALL_SH_URL} | sh -s -- --to {quoted_dir}"
    );
    let spec = CommandSpec::new("sh", &["-c", &script]);
    let manual = match lang {
        Lang::Zh => format!(
            "手动安装：curl --proto '=https' --tlsv1.2 -sSf {JUST_INSTALL_SH_URL} | sh -s -- --to ~/.local/bin"
        ),
        Lang::En => format!(
            "manual install: curl --proto '=https' --tlsv1.2 -sSf {JUST_INSTALL_SH_URL} | sh -s -- --to ~/.local/bin"
        ),
    };

    match env.run_command(&spec) {
        Ok(out) if out.success => {
            let gh_note = record_github_path_note(env, &bin_dir, lang);
            SetupStep {
                id: "just".to_string(),
                category: "tool".to_string(),
                status: SetupStatus::Installed,
                summary: match lang {
                    Lang::Zh => "已安装 just（用户目录免 sudo）".to_string(),
                    Lang::En => "installed just (user directory without sudo)".to_string(),
                },
                detail: Some(join_detail(
                    match lang {
                        Lang::Zh => format!("安装目录：{bin_dir}"),
                        Lang::En => format!("install directory: {bin_dir}"),
                    },
                    gh_note,
                )),
                remedy: None,
            }
        }
        Ok(out) => failed_step(
            "just",
            "tool",
            match lang {
                Lang::Zh => "安装 just 失败".to_string(),
                Lang::En => "failed to install just".to_string(),
            },
            format_cmd_failure(lang, "just install.sh", &out),
            manual,
        ),
        Err(err) => failed_step(
            "just",
            "tool",
            match lang {
                Lang::Zh => "安装 just 失败".to_string(),
                Lang::En => "failed to install just".to_string(),
            },
            format_io_error(lang, "sh", &err),
            manual,
        ),
    }
}

fn execute_just_windows(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    let manual = match lang {
        Lang::Zh => {
            "手动安装命令：winget install Casey.Just --accept-package-agreements --accept-source-agreements"
                .to_string()
        }
        Lang::En => {
            "manual install: winget install Casey.Just --accept-package-agreements --accept-source-agreements"
                .to_string()
        }
    };

    if !env.has_command("winget") {
        return failed_step(
            "just",
            "tool",
            match lang {
                Lang::Zh => "未找到 winget，无法自动安装 just".to_string(),
                Lang::En => "winget not found; cannot automatically install just".to_string(),
            },
            match lang {
                Lang::Zh => {
                    "请先安装 winget 或参考 https://github.com/casey/just#installation 手动安装"
                        .to_string()
                }
                Lang::En => {
                    "install winget first or follow https://github.com/casey/just#installation"
                        .to_string()
                }
            },
            manual,
        );
    }

    let spec = CommandSpec::new(
        "winget",
        &[
            "install",
            "Casey.Just",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ],
    );
    match env.run_command(&spec) {
        Ok(out) if out.success => {
            let bin_dir = windows_winget_links_dir(env);
            let gh_note = bin_dir
                .as_deref()
                .and_then(|d| record_github_path_note(env, d, lang));
            SetupStep {
                id: "just".to_string(),
                category: "tool".to_string(),
                status: SetupStatus::Installed,
                summary: match lang {
                    Lang::Zh => "已通过 winget 安装 just".to_string(),
                    Lang::En => "installed just via winget".to_string(),
                },
                detail: Some(join_detail(
                    match lang {
                        Lang::Zh => "包标识：Casey.Just".to_string(),
                        Lang::En => "package: Casey.Just".to_string(),
                    },
                    gh_note,
                )),
                remedy: None,
            }
        }
        Ok(out) => failed_step(
            "just",
            "tool",
            match lang {
                Lang::Zh => "通过 winget 安装 just 失败".to_string(),
                Lang::En => "failed to install just via winget".to_string(),
            },
            format_cmd_failure(lang, "winget install Casey.Just", &out),
            manual,
        ),
        Err(err) => failed_step(
            "just",
            "tool",
            match lang {
                Lang::Zh => "通过 winget 安装 just 失败".to_string(),
                Lang::En => "failed to install just via winget".to_string(),
            },
            format_io_error(lang, "winget", &err),
            manual,
        ),
    }
}

/// 执行 `pwsh` 步骤：仅 Windows 启用 `just` 层且缺失时尝试通过 `winget` 安装，无 `winget` 则优雅报错。
pub(crate) fn execute_pwsh(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
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

    let manual = match lang {
        Lang::Zh => {
            "手动安装命令：winget install Microsoft.PowerShell --accept-package-agreements --accept-source-agreements"
                .to_string()
        }
        Lang::En => {
            "manual install: winget install Microsoft.PowerShell --accept-package-agreements --accept-source-agreements"
                .to_string()
        }
    };

    if !env.has_command("winget") {
        return failed_step(
            "pwsh",
            "tool",
            match lang {
                Lang::Zh => "未找到 winget，无法自动安装 pwsh".to_string(),
                Lang::En => "winget not found; cannot automatically install pwsh".to_string(),
            },
            match lang {
                Lang::Zh => "Windows 上的 just 配方依赖 PowerShell 7（pwsh）".to_string(),
                Lang::En => "just recipes on Windows require PowerShell 7 (pwsh)".to_string(),
            },
            manual,
        );
    }

    let spec = CommandSpec::new(
        "winget",
        &[
            "install",
            "Microsoft.PowerShell",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ],
    );
    match env.run_command(&spec) {
        Ok(out) if out.success => {
            let pwsh_dir = env
                .env_var("ProgramFiles")
                .filter(|v| !v.trim().is_empty())
                .map(|pf| format!(r"{}\PowerShell\7", pf.trim_end_matches('\\')))
                .unwrap_or_else(|| r"C:\Program Files\PowerShell\7".to_string());
            let gh_note = record_github_path_note(env, &pwsh_dir, lang);
            SetupStep {
                id: "pwsh".to_string(),
                category: "tool".to_string(),
                status: SetupStatus::Installed,
                summary: match lang {
                    Lang::Zh => "已通过 winget 安装 pwsh（PowerShell 7）".to_string(),
                    Lang::En => "installed pwsh (PowerShell 7) via winget".to_string(),
                },
                detail: Some(join_detail(
                    match lang {
                        Lang::Zh => "包标识：Microsoft.PowerShell".to_string(),
                        Lang::En => "package: Microsoft.PowerShell".to_string(),
                    },
                    gh_note,
                )),
                remedy: None,
            }
        }
        Ok(out) => failed_step(
            "pwsh",
            "tool",
            match lang {
                Lang::Zh => "通过 winget 安装 pwsh 失败".to_string(),
                Lang::En => "failed to install pwsh via winget".to_string(),
            },
            format_cmd_failure(lang, "winget install Microsoft.PowerShell", &out),
            manual,
        ),
        Err(err) => failed_step(
            "pwsh",
            "tool",
            match lang {
                Lang::Zh => "通过 winget 安装 pwsh 失败".to_string(),
                Lang::En => "failed to install pwsh via winget".to_string(),
            },
            format_io_error(lang, "winget", &err),
            manual,
        ),
    }
}

/// 执行 `bque` 步骤：Unix 走官方 `install.sh`（musl 静态包），Windows 走官方 `install.ps1`，装后追加 `GITHUB_PATH`。
pub(crate) fn execute_bque(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
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

    match env.target_os() {
        SetupOs::Linux | SetupOs::MacOs => {
            let script = format!("curl -fsSL {BQUE_INSTALL_SH_URL} | sh");
            let spec = CommandSpec::new("sh", &["-c", &script]);
            let manual = match lang {
                Lang::Zh => format!("手动安装命令：{script}"),
                Lang::En => format!("manual install: {script}"),
            };
            match env.run_command(&spec) {
                Ok(out) if out.success => {
                    let bin_dir = unix_local_bin_dir(env);
                    let gh_note = record_github_path_note(env, &bin_dir, lang);
                    SetupStep {
                        id: "bque".to_string(),
                        category: "tool".to_string(),
                        status: SetupStatus::Installed,
                        summary: match lang {
                            Lang::Zh => "已通过官方 install.sh 安装 bque".to_string(),
                            Lang::En => "installed bque via official install.sh".to_string(),
                        },
                        detail: Some(join_detail(
                            match lang {
                                Lang::Zh => format!("安装目录：{bin_dir}"),
                                Lang::En => format!("install directory: {bin_dir}"),
                            },
                            gh_note,
                        )),
                        remedy: None,
                    }
                }
                Ok(out) => failed_step(
                    "bque",
                    "tool",
                    match lang {
                        Lang::Zh => "安装 bque 失败".to_string(),
                        Lang::En => "failed to install bque".to_string(),
                    },
                    format_cmd_failure(lang, "bque install.sh", &out),
                    manual,
                ),
                Err(err) => failed_step(
                    "bque",
                    "tool",
                    match lang {
                        Lang::Zh => "安装 bque 失败".to_string(),
                        Lang::En => "failed to install bque".to_string(),
                    },
                    format_io_error(lang, "sh", &err),
                    manual,
                ),
            }
        }
        SetupOs::Windows => {
            let ps_cmd = format!("irm {BQUE_INSTALL_PS1_URL} | iex");
            let shell = if env.has_command("pwsh") {
                "pwsh"
            } else {
                "powershell"
            };
            let spec = CommandSpec::new(
                shell,
                &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &ps_cmd,
                ],
            );
            let manual = match lang {
                Lang::Zh => format!("手动安装命令：{ps_cmd}"),
                Lang::En => format!("manual install: {ps_cmd}"),
            };
            match env.run_command(&spec) {
                Ok(out) if out.success => {
                    let bin_dir = windows_bque_bin_dir(env);
                    let gh_note = record_github_path_note(env, &bin_dir, lang);
                    SetupStep {
                        id: "bque".to_string(),
                        category: "tool".to_string(),
                        status: SetupStatus::Installed,
                        summary: match lang {
                            Lang::Zh => "已通过官方 install.ps1 安装 bque".to_string(),
                            Lang::En => "installed bque via official install.ps1".to_string(),
                        },
                        detail: Some(join_detail(
                            match lang {
                                Lang::Zh => format!("安装目录：{bin_dir}"),
                                Lang::En => format!("install directory: {bin_dir}"),
                            },
                            gh_note,
                        )),
                        remedy: None,
                    }
                }
                Ok(out) => failed_step(
                    "bque",
                    "tool",
                    match lang {
                        Lang::Zh => "安装 bque 失败".to_string(),
                        Lang::En => "failed to install bque".to_string(),
                    },
                    format_cmd_failure(lang, "bque install.ps1", &out),
                    manual,
                ),
                Err(err) => failed_step(
                    "bque",
                    "tool",
                    match lang {
                        Lang::Zh => "安装 bque 失败".to_string(),
                        Lang::En => "failed to install bque".to_string(),
                    },
                    format_io_error(lang, shell, &err),
                    manual,
                ),
            }
        }
    }
}

/// 执行 `rust` 步骤：缺失 `rustup`/`cargo` 时执行非交互安装；若存在 `rust-toolchain.toml`，
/// 调用 `rustup show active-toolchain` 触发按声明同步 channel/components。
pub(crate) fn execute_rust(root: &Path, env: &dyn SetupEnv, lang: Lang) -> SetupStep {
    let has_rustup = env
        .probe_command_version("rustup", &["--version"])
        .is_some();
    let has_cargo = env.probe_command_version("cargo", &["--version"]).is_some();
    let has_rustc = env.probe_command_version("rustc", &["--version"]).is_some();
    let has_toolchain_file = env.path_exists(&root.join("rust-toolchain.toml"));

    if has_rustup && has_cargo && has_rustc && !has_toolchain_file {
        return SetupStep {
            id: "rust".to_string(),
            category: "tool".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => "Rust 工具链已就绪，跳过".to_string(),
                Lang::En => "Rust toolchain already installed; skipped".to_string(),
            },
            detail: None,
            remedy: None,
        };
    }

    let manual_rustup = match lang {
        Lang::Zh => "手动安装参考：https://rustup.rs".to_string(),
        Lang::En => "manual install: https://rustup.rs".to_string(),
    };

    let mut installed_rustup = false;
    let mut gh_note: Option<String> = None;
    let cargo_bin = cargo_bin_dir(env);

    if !has_rustup || !has_cargo {
        let install_spec = match env.target_os() {
            SetupOs::Linux | SetupOs::MacOs => {
                let script = format!(
                    "curl --proto '=https' --tlsv1.2 -sSf {RUSTUP_INSTALL_SH_URL} | sh -s -- -y"
                );
                CommandSpec::new("sh", &["-c", &script])
            }
            SetupOs::Windows => {
                if !env.has_command("winget") {
                    return failed_step(
                        "rust",
                        "tool",
                        match lang {
                            Lang::Zh => "未找到 winget，无法自动安装 rustup".to_string(),
                            Lang::En => {
                                "winget not found; cannot automatically install rustup".to_string()
                            }
                        },
                        match lang {
                            Lang::Zh => {
                                "请通过 https://rustup.rs 下载 rustup-init.exe 手动安装".to_string()
                            }
                            Lang::En => {
                                "download rustup-init.exe from https://rustup.rs to install manually"
                                    .to_string()
                            }
                        },
                        manual_rustup,
                    );
                }
                CommandSpec::new(
                    "winget",
                    &[
                        "install",
                        "Rustlang.Rustup",
                        "--accept-package-agreements",
                        "--accept-source-agreements",
                    ],
                )
            }
        };

        match env.run_command(&install_spec) {
            Ok(out) if out.success => {
                installed_rustup = true;
                gh_note = record_github_path_note(env, &cargo_bin, lang);
            }
            Ok(out) => {
                return failed_step(
                    "rust",
                    "tool",
                    match lang {
                        Lang::Zh => "安装 rustup 与 Rust 工具链失败".to_string(),
                        Lang::En => "failed to install rustup and Rust toolchain".to_string(),
                    },
                    format_cmd_failure(lang, "rustup install", &out),
                    manual_rustup,
                );
            }
            Err(err) => {
                return failed_step(
                    "rust",
                    "tool",
                    match lang {
                        Lang::Zh => "安装 rustup 与 Rust 工具链失败".to_string(),
                        Lang::En => "failed to install rustup and Rust toolchain".to_string(),
                    },
                    format_io_error(lang, &install_spec.program, &err),
                    manual_rustup,
                );
            }
        }
    }

    let mut synced_toolchain = false;
    if has_toolchain_file {
        let mut sync_spec = CommandSpec::new("rustup", &["show", "active-toolchain"]).cwd(root);
        if installed_rustup {
            if let Some(current_path) = env.env_var("PATH") {
                let sep = match env.target_os() {
                    SetupOs::Windows => ';',
                    SetupOs::Linux | SetupOs::MacOs => ':',
                };
                sync_spec.env.push((
                    "PATH".to_string(),
                    format!("{cargo_bin}{sep}{current_path}"),
                ));
            }
        }
        match env.run_command(&sync_spec) {
            Ok(out) if out.success => {
                synced_toolchain = true;
            }
            Ok(out) => {
                return failed_step(
                    "rust",
                    "tool",
                    match lang {
                        Lang::Zh => "按 rust-toolchain.toml 同步 Rust 工具链失败".to_string(),
                        Lang::En => {
                            "failed to sync Rust toolchain from rust-toolchain.toml".to_string()
                        }
                    },
                    format_cmd_failure(lang, "rustup show active-toolchain", &out),
                    match lang {
                        Lang::Zh => "手动执行：rustup show active-toolchain".to_string(),
                        Lang::En => "manual command: rustup show active-toolchain".to_string(),
                    },
                );
            }
            Err(err) => {
                return failed_step(
                    "rust",
                    "tool",
                    match lang {
                        Lang::Zh => "按 rust-toolchain.toml 同步 Rust 工具链失败".to_string(),
                        Lang::En => {
                            "failed to sync Rust toolchain from rust-toolchain.toml".to_string()
                        }
                    },
                    format_io_error(lang, "rustup", &err),
                    match lang {
                        Lang::Zh => "手动执行：rustup show active-toolchain".to_string(),
                        Lang::En => "manual command: rustup show active-toolchain".to_string(),
                    },
                );
            }
        }
    }

    let (summary, base_detail) = match (installed_rustup, synced_toolchain, lang) {
        (true, true, Lang::Zh) => (
            "已安装 rustup 并按 rust-toolchain.toml 同步工具链与组件".to_string(),
            format!("安装目录：{cargo_bin}；已执行 rustup show active-toolchain"),
        ),
        (true, true, Lang::En) => (
            "installed rustup and synced toolchain/components from rust-toolchain.toml".to_string(),
            format!("install directory: {cargo_bin}; ran rustup show active-toolchain"),
        ),
        (true, false, Lang::Zh) => (
            "已非交互安装 rustup 与 Rust 工具链".to_string(),
            format!("安装目录：{cargo_bin}"),
        ),
        (true, false, Lang::En) => (
            "installed rustup and Rust toolchain non-interactively".to_string(),
            format!("install directory: {cargo_bin}"),
        ),
        (false, _, Lang::Zh) => (
            "已通过 rustup 按 rust-toolchain.toml 同步工具链与组件".to_string(),
            "命令：rustup show active-toolchain".to_string(),
        ),
        (false, _, Lang::En) => (
            "synced Rust toolchain and components from rust-toolchain.toml via rustup".to_string(),
            "command: rustup show active-toolchain".to_string(),
        ),
    };

    SetupStep {
        id: "rust".to_string(),
        category: "tool".to_string(),
        status: SetupStatus::Installed,
        summary,
        detail: Some(join_detail(base_detail, gh_note)),
        remedy: None,
    }
}

/// 受支持的 Linux 系统包管理器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinuxPkgManager {
    Apt,
    Dnf,
    Pacman,
}

impl LinuxPkgManager {
    fn detect(env: &dyn SetupEnv) -> Option<Self> {
        if env.has_command("apt-get") {
            Some(Self::Apt)
        } else if env.has_command("dnf") {
            Some(Self::Dnf)
        } else if env.has_command("pacman") {
            Some(Self::Pacman)
        } else {
            None
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Apt => "apt-get",
            Self::Dnf => "dnf",
            Self::Pacman => "pacman",
        }
    }

    fn manual_command(self) -> &'static str {
        match self {
            Self::Apt => "sudo apt-get update && sudo apt-get install -y lld",
            Self::Dnf => "sudo dnf install -y lld",
            Self::Pacman => "sudo pacman -S --noconfirm lld",
        }
    }
}

/// 执行 `lld` 步骤（Linux + `use_lld`）：先探测免密 `sudo -n`，具备权限时自动识别包管理器静默安装，
/// 无免密权限时绝不卡密码输入，返回 `Failed` 并在 `remedy` 给出明确单行安装命令。
pub(crate) fn execute_lld(env: &dyn SetupEnv, lang: Lang) -> SetupStep {
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

    let pkg_mgr = LinuxPkgManager::detect(env);
    let remedy = match pkg_mgr {
        Some(pm) => match lang {
            Lang::Zh => format!("请手动执行：{}", pm.manual_command()),
            Lang::En => format!("manual install: {}", pm.manual_command()),
        },
        None => match lang {
            Lang::Zh => {
                "请手动执行：sudo apt-get install -y lld（Debian/Ubuntu）、sudo dnf install -y lld（Fedora/RHEL）或 sudo pacman -S --noconfirm lld（Arch）"
                    .to_string()
            }
            Lang::En => {
                "manual install: sudo apt-get install -y lld (Debian/Ubuntu), sudo dnf install -y lld (Fedora/RHEL), or sudo pacman -S --noconfirm lld (Arch)"
                    .to_string()
            }
        },
    };

    if !env.has_passwordless_sudo() {
        return failed_step(
            "lld",
            "tool",
            match lang {
                Lang::Zh => "未检测到免密 sudo 权限，无法自动安装 lld".to_string(),
                Lang::En => {
                    "no passwordless sudo available; cannot automatically install lld".to_string()
                }
            },
            match lang {
                Lang::Zh => "为避免阻塞在密码输入，setup 不会触发交互式 sudo".to_string(),
                Lang::En => {
                    "to avoid blocking on a password prompt, setup never invokes interactive sudo"
                        .to_string()
                }
            },
            remedy,
        );
    }

    let Some(pm) = pkg_mgr else {
        return failed_step(
            "lld",
            "tool",
            match lang {
                Lang::Zh => "未检测到受支持的系统包管理器（apt-get / dnf / pacman），无法自动安装 lld"
                    .to_string(),
                Lang::En => {
                    "no supported system package manager found (apt-get / dnf / pacman); cannot automatically install lld"
                        .to_string()
                }
            },
            match lang {
                Lang::Zh => "请使用当前发行版包管理器安装 lld".to_string(),
                Lang::En => "please install lld with your distribution package manager".to_string(),
            },
            remedy,
        );
    };

    let commands: Vec<CommandSpec> = match pm {
        LinuxPkgManager::Apt => {
            let mut update = CommandSpec::new("sudo", &["-n", "apt-get", "update"]);
            update
                .env
                .push(("DEBIAN_FRONTEND".to_string(), "noninteractive".to_string()));
            let mut install = CommandSpec::new("sudo", &["-n", "apt-get", "install", "-y", "lld"]);
            install
                .env
                .push(("DEBIAN_FRONTEND".to_string(), "noninteractive".to_string()));
            vec![update, install]
        }
        LinuxPkgManager::Dnf => {
            vec![CommandSpec::new(
                "sudo",
                &["-n", "dnf", "install", "-y", "lld"],
            )]
        }
        LinuxPkgManager::Pacman => {
            vec![CommandSpec::new(
                "sudo",
                &["-n", "pacman", "-S", "--noconfirm", "lld"],
            )]
        }
    };

    for spec in &commands {
        let label = format!("{} {}", spec.program, spec.args.join(" "));
        match env.run_command(spec) {
            Ok(out) if out.success => {}
            Ok(out) => {
                return failed_step(
                    "lld",
                    "tool",
                    match lang {
                        Lang::Zh => format!("通过 {} 安装 lld 失败", pm.name()),
                        Lang::En => format!("failed to install lld via {}", pm.name()),
                    },
                    format_cmd_failure(lang, &label, &out),
                    remedy,
                );
            }
            Err(err) => {
                return failed_step(
                    "lld",
                    "tool",
                    match lang {
                        Lang::Zh => format!("通过 {} 安装 lld 失败", pm.name()),
                        Lang::En => format!("failed to install lld via {}", pm.name()),
                    },
                    format_io_error(lang, &label, &err),
                    remedy,
                );
            }
        }
    }

    SetupStep {
        id: "lld".to_string(),
        category: "tool".to_string(),
        status: SetupStatus::Installed,
        summary: match lang {
            Lang::Zh => format!("已通过 {} 安装 lld 链接器（ld.lld）", pm.name()),
            Lang::En => format!("installed lld linker (ld.lld) via {}", pm.name()),
        },
        detail: Some(match lang {
            Lang::Zh => format!("包管理器：{}（免密 sudo）", pm.name()),
            Lang::En => format!("package manager: {} (passwordless sudo)", pm.name()),
        }),
        remedy: None,
    }
}

// ---------------------------------------------------------------------------
// 路径推导与格式化辅助函数
// ---------------------------------------------------------------------------

fn unix_local_bin_dir(env: &dyn SetupEnv) -> String {
    match env.home_dir() {
        Some(home) => {
            let trimmed = home.to_string_lossy().trim_end_matches('/').to_string();
            format!("{trimmed}/.local/bin")
        }
        None => "~/.local/bin".to_string(),
    }
}

fn cargo_bin_dir(env: &dyn SetupEnv) -> String {
    match (env.target_os(), env.home_dir()) {
        (SetupOs::Windows, Some(home)) => {
            let trimmed = home.to_string_lossy().trim_end_matches('\\').to_string();
            format!(r"{trimmed}\.cargo\bin")
        }
        (SetupOs::Windows, None) => r"%USERPROFILE%\.cargo\bin".to_string(),
        (SetupOs::Linux | SetupOs::MacOs, Some(home)) => {
            let trimmed = home.to_string_lossy().trim_end_matches('/').to_string();
            format!("{trimmed}/.cargo/bin")
        }
        (SetupOs::Linux | SetupOs::MacOs, None) => "~/.cargo/bin".to_string(),
    }
}

fn windows_bque_bin_dir(env: &dyn SetupEnv) -> String {
    if let Some(local_app) = env.env_var("LOCALAPPDATA").filter(|v| !v.trim().is_empty()) {
        return format!(r"{}\bque\bin", local_app.trim_end_matches('\\'));
    }
    if let Some(home) = env.home_dir() {
        let trimmed = home.to_string_lossy().trim_end_matches('\\').to_string();
        return format!(r"{trimmed}\AppData\Local\bque\bin");
    }
    r"%LOCALAPPDATA%\bque\bin".to_string()
}

fn windows_winget_links_dir(env: &dyn SetupEnv) -> Option<String> {
    if let Some(local_app) = env.env_var("LOCALAPPDATA").filter(|v| !v.trim().is_empty()) {
        return Some(format!(
            r"{}\Microsoft\WinGet\Links",
            local_app.trim_end_matches('\\')
        ));
    }
    env.home_dir().map(|home| {
        let trimmed = home.to_string_lossy().trim_end_matches('\\').to_string();
        format!(r"{trimmed}\AppData\Local\Microsoft\WinGet\Links")
    })
}

fn shell_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn record_github_path_note(env: &dyn SetupEnv, dir: &str, lang: Lang) -> Option<String> {
    match append_github_path(env, dir) {
        Ok(true) => Some(match lang {
            Lang::Zh => format!("已追加到 GITHUB_PATH：{dir}"),
            Lang::En => format!("appended to GITHUB_PATH: {dir}"),
        }),
        Ok(false) => None,
        Err(err) => Some(match lang {
            Lang::Zh => format!("写入 GITHUB_PATH 失败：{err}"),
            Lang::En => format!("failed to write GITHUB_PATH: {err}"),
        }),
    }
}

fn join_detail(base: String, extra: Option<String>) -> String {
    match extra {
        Some(note) => format!("{base}; {note}"),
        None => base,
    }
}

fn failed_step(
    id: &str,
    category: &str,
    summary: String,
    detail: String,
    remedy: String,
) -> SetupStep {
    SetupStep {
        id: id.to_string(),
        category: category.to_string(),
        status: SetupStatus::Failed,
        summary,
        detail: Some(detail),
        remedy: Some(remedy),
    }
}

fn format_cmd_failure(lang: Lang, label: &str, out: &CommandOutput) -> String {
    let snippet = out
        .stderr
        .lines()
        .chain(out.stdout.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");
    match (lang, snippet.is_empty()) {
        (Lang::Zh, true) => format!("命令 `{label}` 退出码 {}", out.status_code),
        (Lang::Zh, false) => format!("命令 `{label}` 退出码 {}：{snippet}", out.status_code),
        (Lang::En, true) => format!("command `{label}` exited with code {}", out.status_code),
        (Lang::En, false) => {
            format!(
                "command `{label}` exited with code {}: {snippet}",
                out.status_code
            )
        }
    }
}

fn format_io_error(lang: Lang, label: &str, err: &std::io::Error) -> String {
    match lang {
        Lang::Zh => format!("无法启动命令 `{label}`：{err}"),
        Lang::En => format!("failed to invoke `{label}`: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::doctor::absolute;
    use crate::i18n_audit::contains_han;
    use crate::manifest::ProjectManifest;
    use crate::setup::{format_setup_report, run_setup_with_env, FakeSetupEnv, SetupOptions};

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-setup-tools-{}-{}-{}",
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

    #[test]
    fn 已安装工具全部跳过且不执行任何外部命令() {
        let dir = temp_dir("all-installed");
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
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("执行应成功");

        for id in ["just", "bque", "rust", "lld"] {
            let step = report
                .steps
                .iter()
                .find(|s| s.id == id)
                .unwrap_or_else(|| panic!("应包含步骤 {id}"));
            assert_eq!(step.status, SetupStatus::Skipped, "{id} 应跳过");
        }
        assert!(
            env.executed_commands().is_empty(),
            "全部已装时不应执行任何外部命令"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 子模块全部已初始化时标记跳过不执行update() {
        let dir = temp_dir("submodules-clean");
        write_manifest(&dir, &["common", "subrepo"], BTreeMap::new());
        let abs = absolute(&dir).expect("路径归一化失败");

        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_file(abs.join(".gitmodules"), "[submodule \"a\"]\npath = a\n");
        env.push_command_output(CommandOutput::ok(" 1234567890abcdef a (v1.0)\n"));

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("执行应成功");

        let sub = report
            .steps
            .iter()
            .find(|s| s.id == "submodules")
            .expect("应包含 submodules 步骤");
        assert_eq!(sub.status, SetupStatus::Skipped);

        let cmds = env.executed_commands();
        assert_eq!(cmds.len(), 1, "已初始化时只应查询状态，不应执行 update");
        assert_eq!(cmds[0].program, "git");
        assert_eq!(cmds[0].args, vec!["submodule", "status", "--recursive"]);
        assert_eq!(cmds[0].cwd.as_deref(), Some(abs.as_path()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 子模块存在未初始化项时执行递归拉齐() {
        let dir = temp_dir("submodules-init");
        write_manifest(&dir, &["common", "subrepo"], BTreeMap::new());
        let abs = absolute(&dir).expect("路径归一化失败");

        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_file(abs.join(".gitmodules"), "[submodule \"a\"]\npath = a\n");
        env.push_command_output(CommandOutput::ok(
            "-1234567890abcdef a\n 9999999999abcdef b\n",
        ));
        env.push_command_output(CommandOutput::ok("Submodule 'a' registered"));

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("执行应成功");

        let sub = report
            .steps
            .iter()
            .find(|s| s.id == "submodules")
            .expect("应包含 submodules 步骤");
        assert_eq!(sub.status, SetupStatus::Installed);

        let cmds = env.executed_commands();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[1].program, "git");
        assert_eq!(
            cmds[1].args,
            vec!["submodule", "update", "--init", "--recursive"]
        );
        assert_eq!(cmds[1].cwd.as_deref(), Some(abs.as_path()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn linux下安装just与bque并幂等追加github_path() {
        let dir = temp_dir("linux-just-bque");
        write_manifest(&dir, &["common", "just", "bque"], BTreeMap::new());

        let gh_path = PathBuf::from("/tmp/fake-github-path");
        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("GITHUB_PATH", gh_path.to_string_lossy().to_string());

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("执行应成功");

        let just_step = report.steps.iter().find(|s| s.id == "just").expect("just");
        let bque_step = report.steps.iter().find(|s| s.id == "bque").expect("bque");
        assert_eq!(just_step.status, SetupStatus::Installed);
        assert_eq!(bque_step.status, SetupStatus::Installed);

        let cmds = env.executed_commands();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].program, "sh");
        assert!(cmds[0].args[1].contains(JUST_INSTALL_SH_URL));
        assert!(cmds[0].args[1].contains("/home/demo/.local/bin"));
        assert_eq!(cmds[1].program, "sh");
        assert!(cmds[1].args[1].contains(BQUE_INSTALL_SH_URL));

        // 两个工具都装到 /home/demo/.local/bin，GITHUB_PATH 中应只写入一次
        let gh_content = env
            .read_virtual_file(&gh_path)
            .expect("应已写入 GITHUB_PATH");
        assert_eq!(gh_content, "/home/demo/.local/bin\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn windows下安装just与pwsh与bque命令派发() {
        let dir = temp_dir("win-tools");
        write_manifest(&dir, &["common", "just", "bque"], BTreeMap::new());

        let gh_path = PathBuf::from(r"C:\Temp\github_path");
        let env = FakeSetupEnv::new(SetupOs::Windows)
            .with_command("winget", "v1.9.0")
            .with_env("LOCALAPPDATA", r"C:\Users\demo\AppData\Local")
            .with_env("ProgramFiles", r"C:\Program Files")
            .with_env("GITHUB_PATH", gh_path.to_string_lossy().to_string());

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::En),
            },
            &env,
        )
        .expect("Windows 执行应成功");

        for id in ["just", "pwsh", "bque"] {
            let step = report
                .steps
                .iter()
                .find(|s| s.id == id)
                .unwrap_or_else(|| panic!("应包含 {id}"));
            assert_eq!(step.status, SetupStatus::Installed, "{id} 应安装成功");
        }

        let cmds = env.executed_commands();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].program, "winget");
        assert_eq!(
            cmds[0].args,
            vec![
                "install",
                "Casey.Just",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ]
        );
        assert_eq!(cmds[1].program, "winget");
        assert_eq!(
            cmds[1].args,
            vec![
                "install",
                "Microsoft.PowerShell",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ]
        );
        assert_eq!(cmds[2].program, "powershell");
        assert!(cmds[2]
            .args
            .iter()
            .any(|a| a.contains(BQUE_INSTALL_PS1_URL)));

        let gh_content = env
            .read_virtual_file(&gh_path)
            .expect("Windows 应已写入 GITHUB_PATH");
        assert!(gh_content.contains(r"C:\Users\demo\AppData\Local\Microsoft\WinGet\Links"));
        assert!(gh_content.contains(r"C:\Program Files\PowerShell\7"));
        assert!(gh_content.contains(r"C:\Users\demo\AppData\Local\bque\bin"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn windows缺少winget时pwsh与just优雅报错不卡交互() {
        let dir = temp_dir("win-no-winget");
        write_manifest(&dir, &["common", "just"], BTreeMap::new());

        let env = FakeSetupEnv::new(SetupOs::Windows);
        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("执行应返回结构化报告");

        assert_eq!(report.exit_code(), 1);
        let just_step = report.steps.iter().find(|s| s.id == "just").expect("just");
        let pwsh_step = report.steps.iter().find(|s| s.id == "pwsh").expect("pwsh");
        assert_eq!(just_step.status, SetupStatus::Failed);
        assert_eq!(pwsh_step.status, SetupStatus::Failed);
        assert!(just_step
            .remedy
            .as_deref()
            .unwrap_or("")
            .contains("winget install Casey.Just"));
        assert!(pwsh_step
            .remedy
            .as_deref()
            .unwrap_or("")
            .contains("winget install Microsoft.PowerShell"));
        assert!(
            env.executed_commands().is_empty(),
            "缺少 winget 时不应尝试调用不存在的命令"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rust缺失时非交互安装并在存在toolchain文件时同步() {
        let dir = temp_dir("rust-install-and-sync");
        let mut opts = BTreeMap::new();
        opts.insert("use_lld".to_string(), serde_json::json!(false));
        write_manifest(&dir, &["common", "rust"], opts);
        let abs = absolute(&dir).expect("路径归一化失败");

        let gh_path = PathBuf::from("/tmp/rust-gh-path");
        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("GITHUB_PATH", gh_path.to_string_lossy().to_string())
            .with_env("PATH", "/usr/bin:/bin")
            .with_file(
                abs.join("rust-toolchain.toml"),
                "[toolchain]\nchannel = \"1.85.0\"\n",
            );

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("执行应成功");

        let rust_step = report.steps.iter().find(|s| s.id == "rust").expect("rust");
        assert_eq!(rust_step.status, SetupStatus::Installed);

        let cmds = env.executed_commands();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].program, "sh");
        assert!(cmds[0].args[1].contains(RUSTUP_INSTALL_SH_URL));
        assert!(cmds[0].args[1].contains("-y"));
        assert_eq!(cmds[1].program, "rustup");
        assert_eq!(cmds[1].args, vec!["show", "active-toolchain"]);
        assert_eq!(cmds[1].cwd.as_deref(), Some(abs.as_path()));

        let gh_content = env
            .read_virtual_file(&gh_path)
            .expect("应已追加 .cargo/bin 到 GITHUB_PATH");
        assert_eq!(gh_content, "/home/demo/.cargo/bin\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rust已装且存在toolchain文件时仅触发同步() {
        let dir = temp_dir("rust-sync-only");
        let mut opts = BTreeMap::new();
        opts.insert("use_lld".to_string(), serde_json::json!(false));
        write_manifest(&dir, &["common", "rust"], opts);
        let abs = absolute(&dir).expect("路径归一化失败");

        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_command("rustup", "rustup 1.28.1")
            .with_command("cargo", "cargo 1.85.0")
            .with_command("rustc", "rustc 1.85.0")
            .with_file(
                abs.join("rust-toolchain.toml"),
                "[toolchain]\nchannel = \"1.85.0\"\n",
            );

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::En),
            },
            &env,
        )
        .expect("执行应成功");

        let rust_step = report.steps.iter().find(|s| s.id == "rust").expect("rust");
        assert_eq!(rust_step.status, SetupStatus::Installed);

        let cmds = env.executed_commands();
        assert_eq!(cmds.len(), 1, "rustup 已存在时不应重装 rustup");
        assert_eq!(cmds[0].program, "rustup");
        assert_eq!(cmds[0].args, vec!["show", "active-toolchain"]);
        assert_eq!(cmds[0].cwd.as_deref(), Some(abs.as_path()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn linux有免密sudo时自动识别apt_dnf_pacman安装lld() {
        let dir = temp_dir("lld-sudo-pm");
        write_manifest(&dir, &["common", "rust"], BTreeMap::new());

        // 1. apt-get
        let env_apt = FakeSetupEnv::new(SetupOs::Linux)
            .with_command("rustup", "rustup 1.28.1")
            .with_command("cargo", "cargo 1.85.0")
            .with_command("rustc", "rustc 1.85.0")
            .with_command("apt-get", "apt 2.8.0")
            .with_passwordless_sudo(true);
        let r_apt = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env_apt,
        )
        .expect("apt-get 执行应成功");
        let lld_apt = r_apt.steps.iter().find(|s| s.id == "lld").expect("lld");
        assert_eq!(lld_apt.status, SetupStatus::Installed);
        let apt_cmds = env_apt.executed_commands();
        assert_eq!(apt_cmds.len(), 2);
        assert_eq!(apt_cmds[0].program, "sudo");
        assert_eq!(apt_cmds[0].args, vec!["-n", "apt-get", "update"]);
        assert_eq!(apt_cmds[1].program, "sudo");
        assert_eq!(
            apt_cmds[1].args,
            vec!["-n", "apt-get", "install", "-y", "lld"]
        );

        // 2. dnf
        let env_dnf = FakeSetupEnv::new(SetupOs::Linux)
            .with_command("rustup", "rustup 1.28.1")
            .with_command("cargo", "cargo 1.85.0")
            .with_command("rustc", "rustc 1.85.0")
            .with_command("dnf", "4.19.0")
            .with_passwordless_sudo(true);
        let r_dnf = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env_dnf,
        )
        .expect("dnf 执行应成功");
        let lld_dnf = r_dnf.steps.iter().find(|s| s.id == "lld").expect("lld");
        assert_eq!(lld_dnf.status, SetupStatus::Installed);
        let dnf_cmds = env_dnf.executed_commands();
        assert_eq!(dnf_cmds.len(), 1);
        assert_eq!(dnf_cmds[0].program, "sudo");
        assert_eq!(dnf_cmds[0].args, vec!["-n", "dnf", "install", "-y", "lld"]);

        // 3. pacman
        let env_pacman = FakeSetupEnv::new(SetupOs::Linux)
            .with_command("rustup", "rustup 1.28.1")
            .with_command("cargo", "cargo 1.85.0")
            .with_command("rustc", "rustc 1.85.0")
            .with_command("pacman", "Pacman v6.1.0")
            .with_passwordless_sudo(true);
        let r_pacman = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env_pacman,
        )
        .expect("pacman 执行应成功");
        let lld_pacman = r_pacman.steps.iter().find(|s| s.id == "lld").expect("lld");
        assert_eq!(lld_pacman.status, SetupStatus::Installed);
        let pacman_cmds = env_pacman.executed_commands();
        assert_eq!(pacman_cmds.len(), 1);
        assert_eq!(pacman_cmds[0].program, "sudo");
        assert_eq!(
            pacman_cmds[0].args,
            vec!["-n", "pacman", "-S", "--noconfirm", "lld"]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn linux无免密sudo时lld不执行命令并给出单行安装命令() {
        let dir = temp_dir("lld-no-sudo");
        write_manifest(&dir, &["common", "rust"], BTreeMap::new());

        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_command("rustup", "rustup 1.28.1")
            .with_command("cargo", "cargo 1.85.0")
            .with_command("rustc", "rustc 1.85.0")
            .with_command("apt-get", "apt 2.8.0")
            .with_passwordless_sudo(false);

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("执行应返回结构化报告");

        assert_eq!(report.exit_code(), 1);
        let lld_step = report.steps.iter().find(|s| s.id == "lld").expect("lld");
        assert_eq!(lld_step.status, SetupStatus::Failed);
        assert!(
            env.executed_commands().is_empty(),
            "无免密 sudo 时绝不能执行 sudo 阻塞密码输入"
        );
        let remedy = lld_step.remedy.as_deref().unwrap_or("");
        assert!(remedy.contains("sudo apt-get update && sudo apt-get install -y lld"));
        assert!(!remedy.contains('\n'), "remedy 应为明确单行命令");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn github_path幂等追加不重复写入且保留原有条目() {
        let gh_path = PathBuf::from("/tmp/idempotent-gh-path");
        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("GITHUB_PATH", gh_path.to_string_lossy().to_string())
            .with_file(&gh_path, "/existing/bin");

        assert!(append_github_path(&env, "/home/demo/.local/bin").expect("首次追加应成功"));
        assert_eq!(
            env.read_virtual_file(&gh_path).as_deref(),
            Some("/existing/bin\n/home/demo/.local/bin\n")
        );

        // 重复追加同一目录应为幂等空操作
        assert!(!append_github_path(&env, "/home/demo/.local/bin").expect("重复追加不应报错"));
        assert_eq!(
            env.read_virtual_file(&gh_path).as_deref(),
            Some("/existing/bin\n/home/demo/.local/bin\n")
        );

        // 未设置 GITHUB_PATH 时静默跳过
        let no_gh = FakeSetupEnv::new(SetupOs::Linux);
        assert!(!append_github_path(&no_gh, "/home/demo/.local/bin").expect("未设置时应返回 false"));
    }

    #[test]
    fn 真实执行模式下中英双语零汉字混杂() {
        let dir = temp_dir("bilingual-exec");
        write_manifest(
            &dir,
            &["common", "just", "bque", "rust", "subrepo"],
            BTreeMap::new(),
        );
        let abs = absolute(&dir).expect("路径归一化失败");

        let gh_path = PathBuf::from("/tmp/bilingual-gh-path");
        let env_en = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("GITHUB_PATH", gh_path.to_string_lossy().to_string())
            .with_command("apt-get", "apt 2.8.0")
            .with_passwordless_sudo(false)
            .with_file(abs.join(".gitmodules"), "[submodule \"a\"]\npath = a\n")
            .with_file(
                abs.join("rust-toolchain.toml"),
                "[toolchain]\nchannel = \"1.85.0\"\n",
            );
        env_en.push_command_output(CommandOutput::ok("-1234567890abcdef a\n"));

        let report_en = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::En),
            },
            &env_en,
        )
        .expect("英文执行应成功");

        let rendered_en = format_setup_report(&report_en, Lang::En);
        assert!(
            !contains_han(&rendered_en),
            "英文真实执行输出不应含任何汉字:\n{rendered_en}"
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

        // 中文模式应包含中文文案
        let env_zh = FakeSetupEnv::new(SetupOs::Linux)
            .with_command("apt-get", "apt 2.8.0")
            .with_passwordless_sudo(false)
            .with_file(abs.join(".gitmodules"), "[submodule \"a\"]\npath = a\n");
        env_zh.push_command_output(CommandOutput::ok(" 1234567890abcdef a\n"));

        let report_zh = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: Some(false),
                lang: Some(Lang::Zh),
            },
            &env_zh,
        )
        .expect("中文执行应成功");

        let rendered_zh = format_setup_report(&report_zh, Lang::Zh);
        assert!(contains_han(&rendered_zh));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
