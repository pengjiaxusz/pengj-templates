//! 临时受限环境自动识别与无污染优化（`setup` 子模块）。
//!
//! 负责两项职责：
//! 1. 按严格 5 级优先级判定当前是否处于临时受限环境（`--no-ephemeral` > `--ephemeral`/`--ci` >
//!    `CI=true`/`GITHUB_ACTIONS=true` > 磁盘可用 `< 20 GiB` 或总容量 `< 40 GiB` > 默认关闭），
//!    并产出中英双语判定原因说明。
//! 2. 仅通过 [`SetupEnv`] 接缝无污染落盘：优先幂等追加 `CARGO_INCREMENTAL=0` 到 `$GITHUB_ENV`；
//!    否则在启用 `rust`/`rust-workspace` 层时，向用户级 Cargo 配置（优先 `CARGO_HOME/config.toml`，
//!    否则 `<home>/.cargo/config.toml`）结构化合并写入 `[build]\nincremental = false`，
//!    绝不修改项目仓库内的 `.cargo/config.toml` 或 `Cargo.toml`。

use std::path::{Path, PathBuf};

use crate::doctor::{display_path, has_layer};
use crate::i18n_audit::Lang;
use crate::manifest::ProjectManifest;
use crate::setup::{
    SetupEnv, SetupOs, SetupStatus, SetupStep, EPHEMERAL_AVAIL_THRESHOLD_BYTES,
    EPHEMERAL_TOTAL_THRESHOLD_BYTES,
};

/// 临时受限环境优化向 `$GITHUB_ENV` 注入的环境变量键名
const CARGO_INCREMENTAL_KEY: &str = "CARGO_INCREMENTAL";
/// 临时受限环境优化向 `$GITHUB_ENV` 注入的环境变量值
const CARGO_INCREMENTAL_VAL: &str = "0";

/// 按严格 5 级优先级判定当前是否处于临时受限环境，并返回中英双语可读原因：
/// 1. `--no-ephemeral`（`Some(false)`）→ 强制关闭；
/// 2. `--ephemeral` / `--ci`（`Some(true)`）→ 强制开启；
/// 3. `CI=true` 或 `GITHUB_ACTIONS=true` → 自动开启；
/// 4. 工作目录所在分区磁盘可用空间 `< 20 GiB` 或总容量 `< 40 GiB` → 自动开启；
/// 5. 否则关闭（主力开发机默认不触发）。
pub(crate) fn detect_ephemeral(
    root: &Path,
    override_flag: Option<bool>,
    env: &dyn SetupEnv,
    lang: Lang,
) -> (bool, String) {
    if let Some(false) = override_flag {
        return (
            false,
            match lang {
                Lang::Zh => "已通过 --no-ephemeral 显式关闭".to_string(),
                Lang::En => "explicitly disabled via --no-ephemeral".to_string(),
            },
        );
    }

    if let Some(true) = override_flag {
        return (
            true,
            match lang {
                Lang::Zh => "已通过 --ephemeral/--ci 显式开启".to_string(),
                Lang::En => "explicitly enabled via --ephemeral/--ci".to_string(),
            },
        );
    }

    if is_truthy_env(env.env_var("GITHUB_ACTIONS").as_deref()) {
        return (
            true,
            match lang {
                Lang::Zh => "检测到环境变量 GITHUB_ACTIONS=true".to_string(),
                Lang::En => "detected environment variable GITHUB_ACTIONS=true".to_string(),
            },
        );
    }

    if is_truthy_env(env.env_var("CI").as_deref()) {
        return (
            true,
            match lang {
                Lang::Zh => "检测到环境变量 CI=true".to_string(),
                Lang::En => "detected environment variable CI=true".to_string(),
            },
        );
    }

    if let Some(disk) = env.disk_space(root) {
        let avail_gib = bytes_to_gib(disk.available_bytes);
        let total_gib = bytes_to_gib(disk.total_bytes);
        if disk.available_bytes < EPHEMERAL_AVAIL_THRESHOLD_BYTES {
            return (
                true,
                match lang {
                    Lang::Zh => format!("分区可用空间 {avail_gib:.1} GiB 低于 20 GiB 阈值"),
                    Lang::En => {
                        format!(
                            "available disk space {avail_gib:.1} GiB is below the 20 GiB threshold"
                        )
                    }
                },
            );
        }
        if disk.total_bytes < EPHEMERAL_TOTAL_THRESHOLD_BYTES {
            return (
                true,
                match lang {
                    Lang::Zh => format!("分区总容量 {total_gib:.1} GiB 低于 40 GiB 阈值"),
                    Lang::En => {
                        format!(
                            "total disk capacity {total_gib:.1} GiB is below the 40 GiB threshold"
                        )
                    }
                },
            );
        }
        return (
            false,
            match lang {
                Lang::Zh => {
                    format!("磁盘空间充裕（可用 {avail_gib:.1} GiB / 总计 {total_gib:.1} GiB）")
                }
                Lang::En => format!(
                    "sufficient disk space ({avail_gib:.1} GiB available / {total_gib:.1} GiB total)"
                ),
            },
        );
    }

    (
        false,
        match lang {
            Lang::Zh => "未检测到 CI 标记且磁盘空间未触发阈值".to_string(),
            Lang::En => "no CI marker detected and disk threshold not triggered".to_string(),
        },
    )
}

/// 生成 `ephemeral` 装机计划步骤（供 `--dry-run` 与执行模式共享）
pub(crate) fn plan_ephemeral_step(
    has_rust: bool,
    ephemeral: bool,
    ephemeral_reason: &str,
    os: SetupOs,
    env: &dyn SetupEnv,
    lang: Lang,
) -> SetupStep {
    if !ephemeral {
        return SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => format!("非临时受限环境（{ephemeral_reason}），保留默认增量编译"),
                Lang::En => format!(
                    "not an ephemeral environment ({ephemeral_reason}); keeping default incremental builds"
                ),
            },
            detail: None,
            remedy: None,
        };
    }

    if let Some(gh_env_path) = resolve_github_env_path(env) {
        let path_str = display_path(&gh_env_path);
        if env.path_exists(&gh_env_path) {
            if let Ok(existing) = env.read_file(&gh_env_path) {
                if github_env_has_key(&existing, CARGO_INCREMENTAL_KEY) {
                    return SetupStep {
                        id: "ephemeral".to_string(),
                        category: "env".to_string(),
                        status: SetupStatus::Skipped,
                        summary: match lang {
                            Lang::Zh => {
                                format!("$GITHUB_ENV 已包含 {CARGO_INCREMENTAL_KEY}，跳过重复写入")
                            }
                            Lang::En => format!(
                                "$GITHUB_ENV already contains {CARGO_INCREMENTAL_KEY}; skipped"
                            ),
                        },
                        detail: Some(match lang {
                            Lang::Zh => {
                                format!("目标文件：{path_str}；触发原因：{ephemeral_reason}")
                            }
                            Lang::En => {
                                format!("target path: {path_str}; reason: {ephemeral_reason}")
                            }
                        }),
                        remedy: None,
                    };
                }
            }
        }

        return SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Planned,
            summary: match lang {
                Lang::Zh => "临时受限环境优化：向 $GITHUB_ENV 注入 CARGO_INCREMENTAL=0".to_string(),
                Lang::En => {
                    "ephemeral environment optimization: append CARGO_INCREMENTAL=0 to $GITHUB_ENV"
                        .to_string()
                }
            },
            detail: Some(match lang {
                Lang::Zh => format!("目标文件：{path_str}；触发原因：{ephemeral_reason}"),
                Lang::En => format!("target path: {path_str}; reason: {ephemeral_reason}"),
            }),
            remedy: None,
        };
    }

    if !has_rust {
        return SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => {
                    "临时受限环境已激活，但项目未启用 Rust 层且无 $GITHUB_ENV，跳过 Cargo 增量配置"
                        .to_string()
                }
                Lang::En => {
                    "ephemeral environment active, but Rust layer is not enabled and $GITHUB_ENV is unset; skipping Cargo incremental config"
                        .to_string()
                }
            },
            detail: Some(match lang {
                Lang::Zh => format!("触发原因：{ephemeral_reason}"),
                Lang::En => format!("reason: {ephemeral_reason}"),
            }),
            remedy: None,
        };
    }

    let resolved_path = resolve_user_cargo_config_path(env);
    let path_str = match &resolved_path {
        Some(p) => display_path(p),
        None => match os {
            SetupOs::Windows => r"%USERPROFILE%\.cargo\config.toml".to_string(),
            SetupOs::Linux | SetupOs::MacOs => "~/.cargo/config.toml".to_string(),
        },
    };

    if let Some(path) = &resolved_path {
        if env.path_exists(path) {
            if let Ok(existing) = env.read_file(path) {
                if matches!(
                    merge_cargo_disable_incremental(&existing),
                    Ok(CargoIncrementalMerge::AlreadyDisabled)
                ) {
                    return SetupStep {
                        id: "ephemeral".to_string(),
                        category: "env".to_string(),
                        status: SetupStatus::Skipped,
                        summary: match lang {
                            Lang::Zh => {
                                "用户级 Cargo 配置已关闭增量编译（[build] incremental = false），跳过"
                                    .to_string()
                            }
                            Lang::En => {
                                "user Cargo config already disables incremental builds ([build] incremental = false); skipped"
                                    .to_string()
                            }
                        },
                        detail: Some(match lang {
                            Lang::Zh => {
                                format!("目标路径：{path_str}；触发原因：{ephemeral_reason}")
                            }
                            Lang::En => {
                                format!("target path: {path_str}; reason: {ephemeral_reason}")
                            }
                        }),
                        remedy: None,
                    };
                }
            }
        }
    }

    SetupStep {
        id: "ephemeral".to_string(),
        category: "env".to_string(),
        status: SetupStatus::Planned,
        summary: match lang {
            Lang::Zh => {
                "临时受限环境优化：向用户级 Cargo 配置合并写入 [build] incremental = false"
                    .to_string()
            }
            Lang::En => {
                "ephemeral environment optimization: merge [build] incremental = false into user Cargo config"
                    .to_string()
            }
        },
        detail: Some(match lang {
            Lang::Zh => format!(
                "目标路径：{path_str}（绝不修改项目仓库文件）；触发原因：{ephemeral_reason}"
            ),
            Lang::En => format!(
                "target path: {path_str} (never modifies repository files); reason: {ephemeral_reason}"
            ),
        }),
        remedy: None,
    }
}

/// 执行 `ephemeral` 步骤的无污染配置落盘（仅通过 [`SetupEnv`] 读写）
pub(crate) fn execute_ephemeral_step(
    manifest: &ProjectManifest,
    ephemeral_reason: &str,
    env: &dyn SetupEnv,
    lang: Lang,
    step: SetupStep,
) -> SetupStep {
    if step.status != SetupStatus::Planned {
        return step;
    }

    if let Some(gh_env_path) = resolve_github_env_path(env) {
        return execute_github_env_write(&gh_env_path, ephemeral_reason, env, lang);
    }

    let has_rust = has_layer(manifest, "rust") || has_layer(manifest, "rust-workspace");
    if !has_rust {
        return SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => {
                    "临时受限环境已激活，但项目未启用 Rust 层且无 $GITHUB_ENV，跳过 Cargo 增量配置"
                        .to_string()
                }
                Lang::En => {
                    "ephemeral environment active, but Rust layer is not enabled and $GITHUB_ENV is unset; skipping Cargo incremental config"
                        .to_string()
                }
            },
            detail: Some(match lang {
                Lang::Zh => format!("触发原因：{ephemeral_reason}"),
                Lang::En => format!("reason: {ephemeral_reason}"),
            }),
            remedy: None,
        };
    }

    let Some(cargo_config_path) = resolve_user_cargo_config_path(env) else {
        return SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Failed,
            summary: match lang {
                Lang::Zh => "无法定位用户级 Cargo 全局配置文件路径".to_string(),
                Lang::En => "failed to locate user-level Cargo global config path".to_string(),
            },
            detail: Some(match lang {
                Lang::Zh => {
                    format!("未设置 CARGO_HOME 且无法获取用户主目录；触发原因：{ephemeral_reason}")
                }
                Lang::En => format!(
                    "CARGO_HOME is unset and user home directory is unavailable; reason: {ephemeral_reason}"
                ),
            }),
            remedy: Some(match lang {
                Lang::Zh => {
                    "请设置 CARGO_HOME（或 HOME/USERPROFILE）环境变量后重试，或手动导出 CARGO_INCREMENTAL=0"
                        .to_string()
                }
                Lang::En => {
                    "set CARGO_HOME (or HOME/USERPROFILE) and retry, or export CARGO_INCREMENTAL=0 manually"
                        .to_string()
                }
            }),
        };
    };

    execute_user_cargo_config_write(&cargo_config_path, ephemeral_reason, env, lang)
}

fn execute_github_env_write(
    path: &Path,
    ephemeral_reason: &str,
    env: &dyn SetupEnv,
    lang: Lang,
) -> SetupStep {
    let path_str = display_path(path);
    let existing = if env.path_exists(path) {
        match env.read_file(path) {
            Ok(content) => content,
            Err(err) => {
                return SetupStep {
                    id: "ephemeral".to_string(),
                    category: "env".to_string(),
                    status: SetupStatus::Failed,
                    summary: match lang {
                        Lang::Zh => "读取 $GITHUB_ENV 文件失败".to_string(),
                        Lang::En => "failed to read $GITHUB_ENV file".to_string(),
                    },
                    detail: Some(match lang {
                        Lang::Zh => format!("目标文件：{path_str}；错误：{err}"),
                        Lang::En => format!("target path: {path_str}; error: {err}"),
                    }),
                    remedy: Some(match lang {
                        Lang::Zh => {
                            "请确认 $GITHUB_ENV 路径可读可写，或手动在 workflow 中设置 CARGO_INCREMENTAL=0"
                                .to_string()
                        }
                        Lang::En => {
                            "verify $GITHUB_ENV is readable/writable, or set CARGO_INCREMENTAL=0 in the workflow manually"
                                .to_string()
                        }
                    }),
                };
            }
        }
    } else {
        String::new()
    };

    let Some(updated) =
        append_github_env_entry(&existing, CARGO_INCREMENTAL_KEY, CARGO_INCREMENTAL_VAL)
    else {
        return SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => {
                    format!("$GITHUB_ENV 已包含 {CARGO_INCREMENTAL_KEY}，跳过重复写入")
                }
                Lang::En => {
                    format!("$GITHUB_ENV already contains {CARGO_INCREMENTAL_KEY}; skipped")
                }
            },
            detail: Some(match lang {
                Lang::Zh => format!("目标文件：{path_str}；触发原因：{ephemeral_reason}"),
                Lang::En => format!("target path: {path_str}; reason: {ephemeral_reason}"),
            }),
            remedy: None,
        };
    };

    match env.write_file(path, &updated) {
        Ok(()) => SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Installed,
            summary: match lang {
                Lang::Zh => "已向 $GITHUB_ENV 写入 CARGO_INCREMENTAL=0".to_string(),
                Lang::En => "appended CARGO_INCREMENTAL=0 to $GITHUB_ENV".to_string(),
            },
            detail: Some(match lang {
                Lang::Zh => format!("目标文件：{path_str}；触发原因：{ephemeral_reason}"),
                Lang::En => format!("target path: {path_str}; reason: {ephemeral_reason}"),
            }),
            remedy: None,
        },
        Err(err) => SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Failed,
            summary: match lang {
                Lang::Zh => "写入 $GITHUB_ENV 文件失败".to_string(),
                Lang::En => "failed to write $GITHUB_ENV file".to_string(),
            },
            detail: Some(match lang {
                Lang::Zh => format!("目标文件：{path_str}；错误：{err}"),
                Lang::En => format!("target path: {path_str}; error: {err}"),
            }),
            remedy: Some(match lang {
                Lang::Zh => {
                    "请检查 $GITHUB_ENV 文件权限，或手动在 workflow 中设置 CARGO_INCREMENTAL=0"
                        .to_string()
                }
                Lang::En => {
                    "check write permissions for $GITHUB_ENV, or set CARGO_INCREMENTAL=0 in the workflow manually"
                        .to_string()
                }
            }),
        },
    }
}

fn execute_user_cargo_config_write(
    path: &Path,
    ephemeral_reason: &str,
    env: &dyn SetupEnv,
    lang: Lang,
) -> SetupStep {
    let path_str = display_path(path);
    let existing = if env.path_exists(path) {
        match env.read_file(path) {
            Ok(content) => content,
            Err(err) => {
                return SetupStep {
                    id: "ephemeral".to_string(),
                    category: "env".to_string(),
                    status: SetupStatus::Failed,
                    summary: match lang {
                        Lang::Zh => "读取用户级 Cargo 配置文件失败".to_string(),
                        Lang::En => "failed to read user Cargo config file".to_string(),
                    },
                    detail: Some(match lang {
                        Lang::Zh => format!("目标路径：{path_str}；错误：{err}"),
                        Lang::En => format!("target path: {path_str}; error: {err}"),
                    }),
                    remedy: Some(match lang {
                        Lang::Zh => {
                            "请检查用户级 Cargo 配置文件权限，或手动写入 [build] incremental = false"
                                .to_string()
                        }
                        Lang::En => {
                            "check permissions on the user Cargo config file, or add [build] incremental = false manually"
                                .to_string()
                        }
                    }),
                };
            }
        }
    } else {
        String::new()
    };

    match merge_cargo_disable_incremental(&existing) {
        Ok(CargoIncrementalMerge::AlreadyDisabled) => SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Skipped,
            summary: match lang {
                Lang::Zh => {
                    "用户级 Cargo 配置已关闭增量编译（[build] incremental = false），跳过"
                        .to_string()
                }
                Lang::En => {
                    "user Cargo config already disables incremental builds ([build] incremental = false); skipped"
                        .to_string()
                }
            },
            detail: Some(match lang {
                Lang::Zh => format!("目标路径：{path_str}；触发原因：{ephemeral_reason}"),
                Lang::En => format!("target path: {path_str}; reason: {ephemeral_reason}"),
            }),
            remedy: None,
        },
        Ok(CargoIncrementalMerge::Updated(updated)) => match env.write_file(path, &updated) {
            Ok(()) => SetupStep {
                id: "ephemeral".to_string(),
                category: "env".to_string(),
                status: SetupStatus::Installed,
                summary: match lang {
                    Lang::Zh => {
                        "已向用户级 Cargo 配置写入 [build] incremental = false（未修改项目仓库）"
                            .to_string()
                    }
                    Lang::En => {
                        "merged [build] incremental = false into user Cargo config (repository untouched)"
                            .to_string()
                    }
                },
                detail: Some(match lang {
                    Lang::Zh => format!("目标路径：{path_str}；触发原因：{ephemeral_reason}"),
                    Lang::En => format!("target path: {path_str}; reason: {ephemeral_reason}"),
                }),
                remedy: None,
            },
            Err(err) => SetupStep {
                id: "ephemeral".to_string(),
                category: "env".to_string(),
                status: SetupStatus::Failed,
                summary: match lang {
                    Lang::Zh => "写入用户级 Cargo 配置文件失败".to_string(),
                    Lang::En => "failed to write user Cargo config file".to_string(),
                },
                detail: Some(match lang {
                    Lang::Zh => format!("目标路径：{path_str}；错误：{err}"),
                    Lang::En => format!("target path: {path_str}; error: {err}"),
                }),
                remedy: Some(match lang {
                    Lang::Zh => {
                        "请检查目录写权限，或手动在用户级 Cargo 配置中添加 [build] incremental = false"
                            .to_string()
                    }
                    Lang::En => {
                        "check directory write permissions, or add [build] incremental = false to your user Cargo config manually"
                            .to_string()
                    }
                }),
            },
        },
        Err(err) => SetupStep {
            id: "ephemeral".to_string(),
            category: "env".to_string(),
            status: SetupStatus::Failed,
            summary: match lang {
                Lang::Zh => "解析用户级 Cargo 配置文件失败，未做修改".to_string(),
                Lang::En => "failed to parse user Cargo config file; left untouched".to_string(),
            },
            detail: Some(match lang {
                Lang::Zh => format!("目标路径：{path_str}；原因：{}", err.describe(lang)),
                Lang::En => format!("target path: {path_str}; cause: {}", err.describe(lang)),
            }),
            remedy: Some(match lang {
                Lang::Zh => {
                    "请先修复用户级 Cargo 配置文件中的 TOML 语法或 [build] 表定义，再重新运行 setup"
                        .to_string()
                }
                Lang::En => {
                    "fix the TOML syntax or [build] table in your user Cargo config and re-run setup"
                        .to_string()
                }
            }),
        },
    }
}

/// 定位 `$GITHUB_ENV` 文件路径（仅当环境变量存在且非空白时返回）
pub(crate) fn resolve_github_env_path(env: &dyn SetupEnv) -> Option<PathBuf> {
    env.env_var("GITHUB_ENV")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// 定位用户级 Cargo 全局配置文件路径：优先 `CARGO_HOME/config.toml`，否则 `<home>/.cargo/config.toml`
pub(crate) fn resolve_user_cargo_config_path(env: &dyn SetupEnv) -> Option<PathBuf> {
    if let Some(cargo_home) = env
        .env_var("CARGO_HOME")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return Some(PathBuf::from(cargo_home).join("config.toml"));
    }
    env.home_dir()
        .map(|home| home.join(".cargo").join("config.toml"))
}

/// 检查 `$GITHUB_ENV` 文本中是否已存在指定键名（支持 `KEY=VAL` 与 `KEY<<EOF` 多行格式）
fn github_env_has_key(content: &str, key: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return false;
        }
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            rest.starts_with('=') || rest.starts_with("<<")
        } else {
            false
        }
    })
}

/// 幂等向 `$GITHUB_ENV` 内容追加 `KEY=VALUE\n`；若已存在同名键返回 `None`
fn append_github_env_entry(existing: &str, key: &str, value: &str) -> Option<String> {
    if github_env_has_key(existing, key) {
        return None;
    }
    let entry = format!("{key}={value}\n");
    if existing.is_empty() {
        Some(entry)
    } else if existing.ends_with('\n') {
        Some(format!("{existing}{entry}"))
    } else {
        Some(format!("{existing}\n{entry}"))
    }
}

/// 用户级 Cargo 配置结构化合并结果
#[derive(Debug, Clone, PartialEq, Eq)]
enum CargoIncrementalMerge {
    /// 已显式配置 `[build] incremental = false`，无需写盘
    AlreadyDisabled,
    /// 已合并写入 `[build] incremental = false`，携带完整新文本
    Updated(String),
}

/// 用户级 Cargo 配置合并错误
#[derive(Debug, Clone, PartialEq, Eq)]
enum CargoConfigMergeError {
    /// 现有文件内容不是合法 TOML
    InvalidToml(String),
    /// 现有文件中的 `build` 键不是 TOML 表
    BuildNotTable,
    /// 序列化 TOML 失败
    SerializeFailed(String),
}

impl CargoConfigMergeError {
    fn describe(&self, lang: Lang) -> String {
        match (self, lang) {
            (Self::InvalidToml(e), Lang::Zh) => format!("TOML 语法无效（{e}）"),
            (Self::InvalidToml(e), Lang::En) => format!("invalid TOML syntax ({e})"),
            (Self::BuildNotTable, Lang::Zh) => {
                "顶层键 build 不是 TOML 表，无法安全合并 [build]".to_string()
            }
            (Self::BuildNotTable, Lang::En) => {
                "top-level key `build` is not a TOML table; cannot merge [build] safely".to_string()
            }
            (Self::SerializeFailed(e), Lang::Zh) => format!("TOML 序列化失败（{e}）"),
            (Self::SerializeFailed(e), Lang::En) => format!("TOML serialization failed ({e})"),
        }
    }
}

/// 以结构化 TOML 合并方式向用户级 Cargo `config.toml` 幂等写入 `[build]\nincremental = false`，
/// 完整保留用户已有其他配置项与 `[build]` 下的其他键。
fn merge_cargo_disable_incremental(
    existing: &str,
) -> std::result::Result<CargoIncrementalMerge, CargoConfigMergeError> {
    if existing.trim().is_empty() {
        return Ok(CargoIncrementalMerge::Updated(
            "[build]\nincremental = false\n".to_string(),
        ));
    }

    let mut root_table: toml::Table = toml::from_str(existing)
        .map_err(|e| CargoConfigMergeError::InvalidToml(e.message().to_string()))?;

    match root_table.get_mut("build") {
        Some(toml::Value::Table(build_table)) => {
            if build_table.get("incremental").and_then(|v| v.as_bool()) == Some(false) {
                return Ok(CargoIncrementalMerge::AlreadyDisabled);
            }
            build_table.insert("incremental".to_string(), toml::Value::Boolean(false));
        }
        Some(_) => {
            return Err(CargoConfigMergeError::BuildNotTable);
        }
        None => {
            let mut build_table = toml::Table::new();
            build_table.insert("incremental".to_string(), toml::Value::Boolean(false));
            root_table.insert("build".to_string(), toml::Value::Table(build_table));
        }
    }

    let mut rendered = toml::to_string_pretty(&root_table)
        .map_err(|e| CargoConfigMergeError::SerializeFailed(e.to_string()))?;
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(CargoIncrementalMerge::Updated(rendered))
}

fn is_truthy_env(value: Option<&str>) -> bool {
    matches!(
        value.map(|v| v.trim().to_ascii_lowercase()).as_deref(),
        Some("true" | "1" | "yes")
    )
}

fn bytes_to_gib(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::i18n_audit::contains_han;
    use crate::setup::{
        format_setup_report, run_setup_with_env, DiskSpace, FakeSetupEnv, SetupOptions,
    };

    const GIB: u64 = 1024 * 1024 * 1024;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pengj-setup-eph-{}-{}-{}",
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
    fn 临时环境五级判定优先级全覆盖() {
        let root = Path::new("/workspace/demo");

        // 第 1 级：--no-ephemeral 压制一切（CI=true + GITHUB_ACTIONS=true + 极小磁盘）
        let extreme_env = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("CI", "true")
            .with_env("GITHUB_ACTIONS", "true")
            .with_disk_space(Some(DiskSpace {
                available_bytes: 2 * GIB,
                total_bytes: 10 * GIB,
            }));
        let (eph1, reason1_zh) = detect_ephemeral(root, Some(false), &extreme_env, Lang::Zh);
        let (_, reason1_en) = detect_ephemeral(root, Some(false), &extreme_env, Lang::En);
        assert!(!eph1, "--no-ephemeral 必须强制关闭");
        assert!(reason1_zh.contains("--no-ephemeral"));
        assert!(reason1_en.contains("--no-ephemeral"));
        assert!(!contains_han(&reason1_en));

        // 第 2 级：--ephemeral/--ci 压制充裕磁盘与无 CI
        let ample_env = FakeSetupEnv::new(SetupOs::Linux).with_disk_space(Some(DiskSpace {
            available_bytes: 200 * GIB,
            total_bytes: 512 * GIB,
        }));
        let (eph2, reason2_zh) = detect_ephemeral(root, Some(true), &ample_env, Lang::Zh);
        let (_, reason2_en) = detect_ephemeral(root, Some(true), &ample_env, Lang::En);
        assert!(eph2, "--ephemeral/--ci 必须强制开启");
        assert!(reason2_zh.contains("--ephemeral"));
        assert!(reason2_en.contains("--ephemeral"));
        assert!(!contains_han(&reason2_en));

        // 第 3 级：CI=true 与 GITHUB_ACTIONS=true 自动开启（即使磁盘充裕）
        let ci_only = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("CI", "true")
            .with_disk_space(Some(DiskSpace {
                available_bytes: 200 * GIB,
                total_bytes: 512 * GIB,
            }));
        let (eph3a, reason3a_zh) = detect_ephemeral(root, None, &ci_only, Lang::Zh);
        let (_, reason3a_en) = detect_ephemeral(root, None, &ci_only, Lang::En);
        assert!(eph3a);
        assert!(reason3a_zh.contains("CI=true"));
        assert!(reason3a_en.contains("CI=true"));
        assert!(!contains_han(&reason3a_en));

        let gha_only = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("GITHUB_ACTIONS", "true")
            .with_disk_space(Some(DiskSpace {
                available_bytes: 200 * GIB,
                total_bytes: 512 * GIB,
            }));
        let (eph3b, reason3b_zh) = detect_ephemeral(root, None, &gha_only, Lang::Zh);
        let (_, reason3b_en) = detect_ephemeral(root, None, &gha_only, Lang::En);
        assert!(eph3b);
        assert!(reason3b_zh.contains("GITHUB_ACTIONS=true"));
        assert!(reason3b_en.contains("GITHUB_ACTIONS=true"));
        assert!(!contains_han(&reason3b_en));

        // 第 4 级：可用 < 20 GiB 或总容量 < 40 GiB 自动开启（CI=false 不干扰）
        let low_avail = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("CI", "false")
            .with_disk_space(Some(DiskSpace {
                available_bytes: 20 * GIB - 1,
                total_bytes: 100 * GIB,
            }));
        let (eph4a, reason4a_zh) = detect_ephemeral(root, None, &low_avail, Lang::Zh);
        let (_, reason4a_en) = detect_ephemeral(root, None, &low_avail, Lang::En);
        assert!(eph4a, "可用空间低于 20 GiB 应触发");
        assert!(reason4a_zh.contains("20 GiB"));
        assert!(reason4a_en.contains("20 GiB"));
        assert!(!contains_han(&reason4a_en));

        let low_total = FakeSetupEnv::new(SetupOs::Linux).with_disk_space(Some(DiskSpace {
            available_bytes: 25 * GIB,
            total_bytes: 40 * GIB - 1,
        }));
        let (eph4b, reason4b_zh) = detect_ephemeral(root, None, &low_total, Lang::Zh);
        let (_, reason4b_en) = detect_ephemeral(root, None, &low_total, Lang::En);
        assert!(eph4b, "总容量低于 40 GiB 应触发");
        assert!(reason4b_zh.contains("40 GiB"));
        assert!(reason4b_en.contains("40 GiB"));
        assert!(!contains_han(&reason4b_en));

        // 第 5 级：边界值刚好等于 20 GiB 可用 / 40 GiB 总容量，或磁盘探针返回 None → 默认关闭
        let exact_threshold = FakeSetupEnv::new(SetupOs::Linux).with_disk_space(Some(DiskSpace {
            available_bytes: 20 * GIB,
            total_bytes: 40 * GIB,
        }));
        let (eph5a, reason5a_en) = detect_ephemeral(root, None, &exact_threshold, Lang::En);
        assert!(!eph5a, "恰好达到 20 GiB / 40 GiB 阈值不应触发");
        assert!(!contains_han(&reason5a_en));

        let no_disk = FakeSetupEnv::new(SetupOs::Linux).with_disk_space(None);
        let (eph5b, reason5b_zh) = detect_ephemeral(root, None, &no_disk, Lang::Zh);
        let (_, reason5b_en) = detect_ephemeral(root, None, &no_disk, Lang::En);
        assert!(!eph5b, "无 CI 且无磁盘信息时默认关闭");
        assert!(contains_han(&reason5b_zh));
        assert!(!contains_han(&reason5b_en));
    }

    #[test]
    fn github_env幂等追加与重复跳过() {
        let dir = temp_dir("gh-env");
        write_manifest(&dir, &["common", "rust"], BTreeMap::new());

        let gh_env_path = PathBuf::from("/tmp/fake_github_env");
        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("GITHUB_ACTIONS", "true")
            .with_env("GITHUB_ENV", gh_env_path.to_str().unwrap())
            .with_file(&gh_env_path, "RUST_BACKTRACE=1\n");

        // 首次执行：追加 CARGO_INCREMENTAL=0，保留原有项
        let r1 = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: None,
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("首次执行应成功");
        assert!(r1.ephemeral);
        let step1 = r1
            .steps
            .iter()
            .find(|s| s.id == "ephemeral")
            .expect("应含 ephemeral 步骤");
        assert_eq!(step1.status, SetupStatus::Installed);
        assert_eq!(
            env.read_virtual_file(&gh_env_path).as_deref(),
            Some("RUST_BACKTRACE=1\nCARGO_INCREMENTAL=0\n")
        );

        // 二次执行：已存在同名项，幂等跳过且文件内容不变
        let r2 = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: None,
                lang: Some(Lang::En),
            },
            &env,
        )
        .expect("二次执行应成功");
        let step2 = r2
            .steps
            .iter()
            .find(|s| s.id == "ephemeral")
            .expect("应含 ephemeral 步骤");
        assert_eq!(step2.status, SetupStatus::Skipped);
        assert!(!contains_han(&step2.summary));
        assert_eq!(
            env.read_virtual_file(&gh_env_path).as_deref(),
            Some("RUST_BACKTRACE=1\nCARGO_INCREMENTAL=0\n")
        );

        // 绝不触碰项目目录内的 .cargo/config.toml 或 Cargo.toml
        assert!(!dir.join(".cargo/config.toml").exists());
        assert!(!dir.join("Cargo.toml").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 用户级cargo配置首次创建与cargo_home优先() {
        let dir = temp_dir("cargo-create");
        write_manifest(&dir, &["common", "rust-workspace"], BTreeMap::new());

        let cargo_home = PathBuf::from("/opt/custom-cargo-home");
        let expected_config = cargo_home.join("config.toml");

        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_env("CARGO_HOME", cargo_home.to_str().unwrap())
            .with_disk_space(Some(DiskSpace {
                available_bytes: 10 * GIB,
                total_bytes: 30 * GIB,
            }));

        let report = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: None,
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("首次写入用户级 Cargo 配置应成功");

        assert!(report.ephemeral);
        let step = report
            .steps
            .iter()
            .find(|s| s.id == "ephemeral")
            .expect("应含 ephemeral 步骤");
        assert_eq!(step.status, SetupStatus::Installed);

        let written = env
            .read_virtual_file(&expected_config)
            .expect("应写入 CARGO_HOME/config.toml");
        let parsed: toml::Table = toml::from_str(&written).expect("写入内容应为合法 TOML");
        assert_eq!(
            parsed
                .get("build")
                .and_then(|v| v.as_table())
                .and_then(|t| t.get("incremental"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );

        // 未触碰默认 ~/.cargo/config.toml 或项目目录内任何文件
        assert!(env
            .read_virtual_file(&PathBuf::from("/home/demo/.cargo/config.toml"))
            .is_none());
        assert!(!dir.join(".cargo/config.toml").exists());
        assert!(!dir.join("Cargo.toml").exists());
        for p in env.virtual_file_paths() {
            assert!(
                !p.starts_with(&dir),
                "绝不能向项目目录写入任何虚拟文件: {}",
                p.display()
            );
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 用户级cargo配置合并已有build键与幂等跳过() {
        let dir = temp_dir("cargo-merge");
        write_manifest(&dir, &["common", "rust"], BTreeMap::new());

        let user_config = PathBuf::from("/home/demo/.cargo/config.toml");
        let existing_toml = "\
[net]
git-fetch-with-cli = true

[build]
jobs = 4
rustc-wrapper = \"sccache\"
";
        let env = FakeSetupEnv::new(SetupOs::Linux)
            .with_file(&user_config, existing_toml)
            .with_disk_space(Some(DiskSpace {
                available_bytes: 15 * GIB,
                total_bytes: 64 * GIB,
            }));

        // 首次执行：合并写入 incremental = false，完整保留已有 [net] 与 [build] 键
        let r1 = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: None,
                lang: Some(Lang::En),
            },
            &env,
        )
        .expect("合并写入应成功");
        let step1 = r1
            .steps
            .iter()
            .find(|s| s.id == "ephemeral")
            .expect("应含 ephemeral 步骤");
        assert_eq!(step1.status, SetupStatus::Installed);
        assert!(!contains_han(&step1.summary));
        assert!(!contains_han(step1.detail.as_deref().unwrap_or("")));

        let merged = env
            .read_virtual_file(&user_config)
            .expect("应存在合并后的用户级 Cargo 配置");
        let parsed: toml::Table = toml::from_str(&merged).expect("合并后应为合法 TOML");
        assert_eq!(
            parsed
                .get("net")
                .and_then(|v| v.as_table())
                .and_then(|t| t.get("git-fetch-with-cli"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        let build = parsed
            .get("build")
            .and_then(|v| v.as_table())
            .expect("应包含 [build] 表");
        assert_eq!(build.get("jobs").and_then(|v| v.as_integer()), Some(4));
        assert_eq!(
            build.get("rustc-wrapper").and_then(|v| v.as_str()),
            Some("sccache")
        );
        assert_eq!(
            build.get("incremental").and_then(|v| v.as_bool()),
            Some(false)
        );

        // 二次执行：已存在 incremental = false → 标记 Skipped
        let r2 = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: None,
                lang: Some(Lang::Zh),
            },
            &env,
        )
        .expect("二次执行应成功");
        let step2 = r2
            .steps
            .iter()
            .find(|s| s.id == "ephemeral")
            .expect("应含 ephemeral 步骤");
        assert_eq!(step2.status, SetupStatus::Skipped);
        assert_eq!(
            env.read_virtual_file(&user_config).as_deref(),
            Some(merged.as_str())
        );

        // 绝不触碰项目目录
        assert!(!dir.join(".cargo/config.toml").exists());
        assert!(!dir.join("Cargo.toml").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 报告中英双语零汉字混杂与损坏toml优雅失败() {
        let dir = temp_dir("bilingual-and-error");
        write_manifest(&dir, &["common", "rust"], BTreeMap::new());

        let user_config = PathBuf::from("/home/demo/.cargo/config.toml");
        let broken_env = FakeSetupEnv::new(SetupOs::Linux)
            .with_file(&user_config, "[build\nbroken = ")
            .with_env("CI", "true");

        let report_en = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: None,
                lang: Some(Lang::En),
            },
            &broken_env,
        )
        .expect("损坏 TOML 应返回结构化报告而非 panic");
        assert_eq!(report_en.exit_code(), 1);
        let eph_step = report_en
            .steps
            .iter()
            .find(|s| s.id == "ephemeral")
            .expect("应含 ephemeral 步骤");
        assert_eq!(eph_step.status, SetupStatus::Failed);

        let rendered_en = format_setup_report(&report_en, Lang::En);
        assert!(
            !contains_han(&rendered_en),
            "英文模式报告不应含任何汉字:\n{rendered_en}"
        );
        // 原始损坏文件不得被覆盖
        assert_eq!(
            broken_env.read_virtual_file(&user_config).as_deref(),
            Some("[build\nbroken = ")
        );

        // 缺失 home_dir 且无 CARGO_HOME 时优雅失败且英文零汉字
        let no_home_env = FakeSetupEnv::new(SetupOs::Linux)
            .with_home_dir(None)
            .with_env("CI", "true");
        let report_no_home = run_setup_with_env(
            &dir,
            &SetupOptions {
                dry_run: false,
                ephemeral: None,
                lang: Some(Lang::En),
            },
            &no_home_env,
        )
        .expect("缺失主目录应返回结构化失败");
        assert_eq!(report_no_home.exit_code(), 1);
        let rendered_no_home = format_setup_report(&report_no_home, Lang::En);
        assert!(!contains_han(&rendered_no_home));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
