# {% if options["skill_lang"] == "en" %}just/bque.just — global cross-worktree build queue (managed by the bque layer; template updates overwrite this file){% else %}just/bque.just — 全局跨工作树编译排队（bque 层托管，模板更新整页覆盖）{% endif %}
# {% if options["skill_lang"] == "en" %}bque is a Windows global kernel lock scheduler: the unix branch does not queue and only passes the command through.{% else %}bque 是 Windows 全局内核锁调度器：unix 分支不排队，只做等价透传与提示。{% endif %}
# {% if options["skill_lang"] == "en" %}Every heavy command should go through q-run / q-bench to avoid saturating the CPU and polluting benchmarks.{% else %}任何重型命令都应经 q-run / q-bench，避免裸跑吃满 CPU 与污染跑分。{% endif %}

# {% if options["skill_lang"] == "en" %}Inspect the global cross-worktree queue (lock holder / waiters / CPU budget){% else %}查看全局跨工作树排队状态（持锁者 / 排队者 / CPU 预算）{% endif %}
[windows]
queue-status:
    @bque status
[unix]
queue-status:
    @echo "bque is Windows-only: no global build queue on this platform"

# {% if options["skill_lang"] == "en" %}Emergency cleanup of stale build locks and orphaned waiter records{% else %}应急清理异常残留的编译锁与孤儿等待记录{% endif %}
[windows]
queue-clean:
    @bque clean
[unix]
queue-clean:
    @echo "bque is Windows-only: no global build lock to clean"

# {% if options["skill_lang"] == "en" %}Manually pause background builds (for gaming / interactive profiling / system-wide tracing){% else %}手动暂停后台编译（让路游戏 / 交互压测 / 系统级画像）{% endif %}
[windows]
queue-pause:
    @bque pause
[unix]
queue-pause:
    @echo "bque is Windows-only: nothing to pause"

# {% if options["skill_lang"] == "en" %}Manually resume background builds{% else %}手动恢复后台编译{% endif %}
[windows]
queue-resume:
    @bque resume
[unix]
queue-resume:
    @echo "bque is Windows-only: nothing to resume"

# {% if options["skill_lang"] == "en" %}Environment check: probe bque on PATH, fall back to the default install directory, and print a one-line installer when missing{% else %}环境自检：探测 PATH 中的 bque，回退到默认安装目录，缺失时给出一键安装命令{% endif %}
[windows]
queue-doctor:
    @$cmd = Get-Command bque -ErrorAction SilentlyContinue; if (-not $cmd -and (Test-Path "$env:LOCALAPPDATA\bque\bin\bque.exe")) { $env:Path = "$env:LOCALAPPDATA\bque\bin;$env:Path"; $cmd = Get-Command bque -ErrorAction SilentlyContinue }; if ($cmd) { Write-Host "bque: $($cmd.Source)"; bque --version; bque status } else { Write-Host 'bque missing. Install: irm https://github.com/pengjiaxusz/bque/releases/latest/download/install.ps1 | iex' }
[unix]
queue-doctor:
    @echo "bque is Windows-only: no installation check needed"

# {% if options["skill_lang"] == "en" %}Usage: just q-run cargo build --release (do not add `--`; it becomes part of the command){% else %}用法：just q-run cargo build --release（不要加 `--`，它会成为命令的一部分）{% endif %}
# {% if options["skill_lang"] == "en" %}Generic queued runner: put any heavy command under the global lock + Idle priority{% else %}通用排队执行入口：把任意重型命令纳入全局锁 + Idle 优先级{% endif %}
[windows]
q-run *ARGS:
    @bque run -- {% raw %}{{ARGS}}{% endraw %}
[unix]
q-run *ARGS:
    {% raw %}{{ARGS}}{% endraw %}

# {% if options["skill_lang"] == "en" %}Usage: just q-bench cargo bench --workspace (do not add `--`){% else %}用法：just q-bench cargo bench --workspace（不要加 `--`）{% endif %}
# {% if options["skill_lang"] == "en" %}Clean-room benchmark: exclusive lock + second-level suspension of background builds for 0% CPU interference{% else %}纯净跑分入口：独占排他锁 + 秒级挂起后台编译，保证 0% CPU 干扰{% endif %}
[windows]
q-bench *ARGS:
    @bque bench -- {% raw %}{{ARGS}}{% endraw %}
[unix]
q-bench *ARGS:
    {% raw %}{{ARGS}}{% endraw %}
