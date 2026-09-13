<#
.SYNOPSIS
    Worktree 感知的极速线性化分支同步脚本 (Branch Sync Fast-Track)

.DESCRIPTION
    自动化执行集成分支与特性分支的线性同步流程：
    1. 自动探测单仓库 / 多 Worktree 模式及分支占用状态（判定 Route A 自由分支 vs Route B Worktree 占用）；
    2. 严格核验工作区干净度，拦截脏改动风险（智能识别并排除内置 Worktree 目录）；
    3. 基于 Git 原生 patch-id (git cherry -v) 毫秒级甄别净新增提交，自动剔除同名/同改动重复提交；
    4. 默认预检模式 (Dry Run)，显示紧凑摘要；
    5. 指定 -Apply 时一键管道化执行变基/快进合并或 cherry-pick、安全同步被合并分支、--force-with-lease 推送并自动完成合后校验。

.PARAMETER SourceBranch
    待合入的源特性分支名（例如: feat/foo）。必须提供。

.PARAMETER IntegrationBranch
    目标集成分支名。默认自动探测 origin/main -> origin/master -> main -> master。

.PARAMETER Apply
    是否执行实际合并、推送与对齐。未指定时仅进行快速预检 (Dry Run) 并输出紧凑报告。

.PARAMETER NoPush
    在 -Apply 执行时不进行 git push（用于本地演练或离线环境）。

.PARAMETER NoFetch
    执行前跳过 git fetch。

.EXAMPLE
    # 快速预检：1 秒内完成拓扑识别与净贡献甄别，输出紧凑摘要
    pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch feat/my-feature

    # 一键执行：自动化完成线性合入、分支对齐、安全推送与合后自检
    pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch feat/my-feature -Apply
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$SourceBranch,

    [Parameter(Position = 1)]
    [string]$IntegrationBranch = "",

    [switch]$Apply,
    [switch]$NoPush,
    [switch]$NoFetch
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

function Invoke-Git {
    param(
        [string[]]$CommandArgs,
        [string]$WorkingDir = ""
    )
    $pinfo = New-Object System.Diagnostics.ProcessStartInfo
    $pinfo.FileName = "git"
    $pinfo.Arguments = ($CommandArgs -join " ")
    $pinfo.RedirectStandardOutput = $true
    $pinfo.RedirectStandardError = $true
    $pinfo.UseShellExecute = $false
    $pinfo.CreateNoWindow = $true
    $pinfo.StandardOutputEncoding = [System.Text.Encoding]::UTF8
    $pinfo.StandardErrorEncoding = [System.Text.Encoding]::UTF8
    if ($WorkingDir) {
        $pinfo.WorkingDirectory = $WorkingDir
    }

    $process = [System.Diagnostics.Process]::Start($pinfo)
    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()

    return [PSCustomObject]@{
        ExitCode = $process.ExitCode
        Output   = $stdout.Trim()
        Error    = $stderr.Trim()
    }
}

function Assert-GitSuccess {
    param(
        [object]$Result,
        [string]$StepName
    )
    if ($Result.ExitCode -ne 0) {
        $msg = if ($Result.Error) { $Result.Error } else { $Result.Output }
        throw "[$StepName] Git 命令执行失败 (退出码 $($Result.ExitCode)): $msg"
    }
}

# 1. 规范化分支名（去除 refs/heads/ 前缀与首尾空格）
$SourceBranch = $SourceBranch.Trim().Replace("refs/heads/", "")
if ([string]::IsNullOrWhiteSpace($SourceBranch)) {
    throw "必须通过 -SourceBranch 指定有效的源分支名。"
}

# 检查是否存在 remote origin
$remoteCheck = Invoke-Git @("remote", "get-url", "origin")
$hasRemote = ($remoteCheck.ExitCode -eq 0)

# 2. 自动探测集成分支
if ([string]::IsNullOrWhiteSpace($IntegrationBranch)) {
    if ($hasRemote) {
        $rMain = Invoke-Git @("rev-parse", "--verify", "origin/main")
        if ($rMain.ExitCode -eq 0) {
            $IntegrationBranch = "main"
        } else {
            $rMaster = Invoke-Git @("rev-parse", "--verify", "origin/master")
            if ($rMaster.ExitCode -eq 0) {
                $IntegrationBranch = "master"
            }
        }
    }
    if ([string]::IsNullOrWhiteSpace($IntegrationBranch)) {
        $lMain = Invoke-Git @("rev-parse", "--verify", "main")
        if ($lMain.ExitCode -eq 0) {
            $IntegrationBranch = "main"
        } else {
            $IntegrationBranch = "master"
        }
    }
}

# 3. 校验分支存在性
$sourceCheck = Invoke-Git @("rev-parse", "--verify", $SourceBranch)
if ($sourceCheck.ExitCode -ne 0) {
    if ($hasRemote) {
        $sourceRemoteCheck = Invoke-Git @("rev-parse", "--verify", "origin/$SourceBranch")
        if ($sourceRemoteCheck.ExitCode -ne 0) {
            throw "源分支 '$SourceBranch' 在本地及远端 origin 均不存在，请检查分支名称。"
        }
    } else {
        throw "源分支 '$SourceBranch' 在本地不存在，请检查分支名称。"
    }
}

$integCheck = Invoke-Git @("rev-parse", "--verify", $IntegrationBranch)
if ($integCheck.ExitCode -ne 0) {
    throw "集成分支 '$IntegrationBranch' 不存在，请检查指定分支名称。"
}

# 4. 获取所有 Worktree 拓扑，判定分支占用状态
$worktreeRaw = Invoke-Git @("worktree", "list", "--porcelain")
Assert-GitSuccess $worktreeRaw "获取 worktree 列表"

$currentWorktreePath = (Get-Location).Path
try {
    $currentWorktreePath = (Resolve-Path $currentWorktreePath).Path
} catch {}

$occupiedWorktreePath = $null
$isOccupiedByOther = $false
$allWorktreePaths = [System.Collections.Generic.List[string]]::new()

$lines = $worktreeRaw.Output -split "`r?`n"
$tempPath = ""
$tempBranch = ""

foreach ($line in $lines) {
    if ($line.StartsWith("worktree ")) {
        $tempPath = $line.Substring(9).Trim()
        try {
            $tempPath = (Resolve-Path $tempPath).Path
        } catch {}
        $allWorktreePaths.Add($tempPath)
    } elseif ($line.StartsWith("branch refs/heads/")) {
        $tempBranch = $line.Substring(18).Trim()
        if ($tempBranch -eq $SourceBranch) {
            if ($tempPath -ne $currentWorktreePath) {
                $isOccupiedByOther = $true
                $occupiedWorktreePath = $tempPath
            }
        }
    } elseif ([string]::IsNullOrWhiteSpace($line)) {
        $tempPath = ""
        $tempBranch = ""
    }
}

$route = if ($isOccupiedByOther) { "Route B" } else { "Route A" }

# 5. 过滤并检查当前工作区干净度（排除本身是已登记 worktree 的未追踪项）
$rootStatus = Invoke-Git @("status", "--porcelain")
if ($rootStatus.ExitCode -ne 0) {
    throw "无法获取当前 Git 仓库状态: $($rootStatus.Error)"
}

$dirtyItems = [System.Collections.Generic.List[string]]::new()
if (-not [string]::IsNullOrWhiteSpace($rootStatus.Output)) {
    $statusLines = $rootStatus.Output -split "`r?`n"
    foreach ($sl in $statusLines) {
        if ([string]::IsNullOrWhiteSpace($sl)) { continue }
        $code = $sl.Substring(0, 2)
        $relPath = $sl.Substring(3).Trim().Trim('"').TrimEnd('/')
        $absCandidate = Join-Path $currentWorktreePath $relPath
        try {
            if (Test-Path $absCandidate) {
                $absCandidate = (Resolve-Path $absCandidate).Path
            }
        } catch {}

        # 若是已登记的 worktree 目录，予以忽略
        $isRegisteredWt = $false
        foreach ($wt in $allWorktreePaths) {
            if ($wt -eq $absCandidate) {
                $isRegisteredWt = $true
                break
            }
        }
        if (-not $isRegisteredWt) {
            $dirtyItems.Add($sl)
        }
    }
}

if ($dirtyItems.Count -gt 0) {
    $dirtyList = $dirtyItems -join "`n"
    throw "当前工作区存在未提交改动，请先提交或执行 git stash 暂存后再运行同步：`n$dirtyList"
}

# 若为 Route B，额外检查目标 worktree 干净度
if ($route -eq "Route B") {
    $wtStatus = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "status", "--porcelain")
    if ($wtStatus.ExitCode -ne 0) {
        throw "无法检查占用分支的 Worktree ($occupiedWorktreePath) 状态: $($wtStatus.Error)"
    }
    if (-not [string]::IsNullOrWhiteSpace($wtStatus.Output)) {
        throw "占用源分支 '$SourceBranch' 的 Worktree ($occupiedWorktreePath) 存在未提交改动，严禁强制重置！请先在该目录提交或 stash。"
    }
}

# 6. Fetch 远端最新状态
if ($hasRemote -and (-not $NoFetch) -and (-not $NoPush)) {
    Write-Host "正在拉取远端最新引用 (git fetch)..." -ForegroundColor DarkGray
    Invoke-Git @("fetch", "origin", $IntegrationBranch) | Out-Null
    Invoke-Git @("fetch", "origin", $SourceBranch) | Out-Null
}

# 7. 净贡献自动甄别 (基于 git cherry -v 毫秒级 patch 匹配)
$cherryRes = Invoke-Git @("cherry", "-v", $IntegrationBranch, $SourceBranch)
Assert-GitSuccess $cherryRes "执行 git cherry 分析"

$netCommits = [System.Collections.Generic.List[PSCustomObject]]::new()
$duplicateCommits = [System.Collections.Generic.List[PSCustomObject]]::new()

if (-not [string]::IsNullOrWhiteSpace($cherryRes.Output)) {
    $cherryLines = $cherryRes.Output -split "`r?`n"
    foreach ($cl in $cherryLines) {
        if ($cl -match '^\+\s+([0-9a-fA-F]+)\s+(.*)$') {
            $netCommits.Add([PSCustomObject]@{
                Hash    = $matches[1]
                Subject = $matches[2]
            })
        } elseif ($cl -match '^\-\s+([0-9a-fA-F]+)\s+(.*)$') {
            $duplicateCommits.Add([PSCustomObject]@{
                Hash    = $matches[1]
                Subject = $matches[2]
            })
        }
    }
}

# 读取当前 HEAD commit
$integCommit = (Invoke-Git @("rev-parse", "--short", $IntegrationBranch)).Output
$sourceCommit = (Invoke-Git @("rev-parse", "--short", $SourceBranch)).Output

# 8. 模式分流：预检报告 (Dry Run)
if (-not $Apply) {
    Write-Host ""
    Write-Host "================== Branch Sync 预检报告 ==================" -ForegroundColor Cyan
    Write-Host "集成分支: $IntegrationBranch ($integCommit)"
    Write-Host "源分支:   $SourceBranch ($sourceCommit)"
    if ($route -eq "Route A") {
        Write-Host "同步路径: Route A [自由分支 / 单仓库模式] (支持 direct rebase + ff-merge)" -ForegroundColor Green
    } else {
        Write-Host "同步路径: Route B [Worktree 占用模式: $occupiedWorktreePath]" -ForegroundColor Yellow
    }
    Write-Host "工作区:   干净 (Clean)" -ForegroundColor Green
    Write-Host "----------------------------------------------------------"
    Write-Host "净贡献提交分析:" -ForegroundColor Cyan
    if ($netCommits.Count -eq 0) {
        Write-Host "  无净新增提交（源分支已包含在集成分支或全部改动已合入）。" -ForegroundColor Yellow
    } else {
        Write-Host "  发现 $($netCommits.Count) 个净新增提交（按正序排列）：" -ForegroundColor Green
        foreach ($nc in $netCommits) {
            Write-Host "    + $($nc.Hash.Substring(0, [Math]::Min(7, $nc.Hash.Length))) $($nc.Subject)" -ForegroundColor White
        }
    }
    if ($duplicateCommits.Count -gt 0) {
        Write-Host "  自动忽略 $($duplicateCommits.Count) 个已包含相同改动的等价提交 (同 patch-id)：" -ForegroundColor DarkGray
        foreach ($dc in $duplicateCommits) {
            Write-Host "    - $($dc.Hash.Substring(0, [Math]::Min(7, $dc.Hash.Length))) $($dc.Subject)" -ForegroundColor DarkGray
        }
    }
    Write-Host "----------------------------------------------------------"
    Write-Host "操作指引:" -ForegroundColor Cyan
    Write-Host "  若确认无误，请运行带 -Apply 参数的一键合入命令："
    Write-Host "  pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch '$SourceBranch' -Apply" -ForegroundColor Yellow
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host ""
    return
}

# 9. 执行阶段 (Apply)
Write-Host ""
Write-Host ">> 开始执行分支同步 (路径: $route)..." -ForegroundColor Cyan

# 记录开始时的当前所在分支，以便必要时恢复
$initialBranch = (Invoke-Git @("rev-parse", "--abbrev-ref", "HEAD")).Output
$doPush = $hasRemote -and (-not $NoPush)

# 目标对齐点（有 push 则 origin/$IntegrationBranch，否则 $IntegrationBranch）
$alignTarget = if ($doPush) { "origin/$IntegrationBranch" } else { $IntegrationBranch }

if ($netCommits.Count -eq 0) {
    Write-Host "提示: 源分支无净新增提交，直接执行对齐同步..." -ForegroundColor Yellow
    if ($route -eq "Route A") {
        Invoke-Git @("checkout", $SourceBranch) | Out-Null
        $res = Invoke-Git @("reset", "--hard", $IntegrationBranch)
        Assert-GitSuccess $res "重置 $SourceBranch 到 $IntegrationBranch"
        if ($doPush) {
            Write-Host "推送对齐分支 origin/$SourceBranch..." -ForegroundColor DarkGray
            $pushRes = Invoke-Git @("push", "--force-with-lease", "origin", $SourceBranch)
            Assert-GitSuccess $pushRes "推送 $SourceBranch"
        }
        Invoke-Git @("checkout", $IntegrationBranch) | Out-Null
    } else {
        $res = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "reset", "--hard", $alignTarget)
        Assert-GitSuccess $res "重置 Worktree ($occupiedWorktreePath) 到 $alignTarget"
        if ($doPush) {
            Write-Host "推送对齐分支 origin/$SourceBranch..." -ForegroundColor DarkGray
            $pushRes = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "push", "--force-with-lease", "origin", $SourceBranch)
            Assert-GitSuccess $pushRes "Worktree 推送 $SourceBranch"
        }
    }
} else {
    if ($route -eq "Route A") {
        # Route A 流程: checkout feat/x -> rebase integration -> checkout integration -> merge --ff-only -> push -> reset feat/x -> push feat/x
        Write-Host "1. 切换至源分支 '$SourceBranch' 并执行变基 (rebase $IntegrationBranch)..." -ForegroundColor DarkGray
        $co1 = Invoke-Git @("checkout", $SourceBranch)
        Assert-GitSuccess $co1 "切换至 $SourceBranch"

        $rb = Invoke-Git @("rebase", $IntegrationBranch)
        if ($rb.ExitCode -ne 0) {
            Write-Host "变基过程发生冲突！正在中止变基并恢复现场..." -ForegroundColor Red
            Invoke-Git @("rebase", "--abort") | Out-Null
            Invoke-Git @("checkout", $initialBranch) | Out-Null
            throw "在对 '$SourceBranch' 执行 'git rebase $IntegrationBranch' 时遇到冲突。请手动解决冲突后再次执行。"
        }

        Write-Host "2. 切换回集成分支 '$IntegrationBranch' 并执行快进合并 (merge --ff-only)..." -ForegroundColor DarkGray
        $co2 = Invoke-Git @("checkout", $IntegrationBranch)
        Assert-GitSuccess $co2 "切换至 $IntegrationBranch"

        $mg = Invoke-Git @("merge", "--ff-only", $SourceBranch)
        Assert-GitSuccess $mg "快进合并 $SourceBranch"

        if ($doPush) {
            Write-Host "3. 推送集成分支 origin/$IntegrationBranch..." -ForegroundColor DarkGray
            $pInteg = Invoke-Git @("push", "origin", $IntegrationBranch)
            Assert-GitSuccess $pInteg "推送集成分支 $IntegrationBranch"
        }

        Write-Host "4. 同步并对齐源分支 '$SourceBranch'..." -ForegroundColor DarkGray
        $co3 = Invoke-Git @("checkout", $SourceBranch)
        Assert-GitSuccess $co3 "切换至 $SourceBranch 进行对齐"

        $rst = Invoke-Git @("reset", "--hard", $IntegrationBranch)
        Assert-GitSuccess $rst "重置 $SourceBranch 到 $IntegrationBranch"

        if ($doPush) {
            Write-Host "5. 推送对齐后的源分支 origin/$SourceBranch (--force-with-lease)..." -ForegroundColor DarkGray
            $pSrc = Invoke-Git @("push", "--force-with-lease", "origin", $SourceBranch)
            Assert-GitSuccess $pSrc "推送源分支 $SourceBranch"
        }

        # 切回集成分支
        Invoke-Git @("checkout", $IntegrationBranch) | Out-Null

    } else {
        # Route B 流程: 主仓库 cherry-pick -> push integration -> worktree reset & push
        Write-Host "1. 切换至集成分支 '$IntegrationBranch' 并按序 cherry-pick 净贡献提交..." -ForegroundColor DarkGray
        $co = Invoke-Git @("checkout", $IntegrationBranch)
        Assert-GitSuccess $co "切换至 $IntegrationBranch"

        $hashList = $netCommits | ForEach-Object { $_.Hash }
        $cpArgs = @("cherry-pick") + $hashList
        $cp = Invoke-Git $cpArgs
        if ($cp.ExitCode -ne 0) {
            Write-Host "Cherry-pick 过程发生冲突！正在中止 cherry-pick..." -ForegroundColor Red
            Invoke-Git @("cherry-pick", "--abort") | Out-Null
            throw "Cherry-pick 净贡献提交时发生冲突。请手动检查并合入。"
        }

        if ($doPush) {
            Write-Host "2. 推送集成分支 origin/$IntegrationBranch..." -ForegroundColor DarkGray
            $pInteg = Invoke-Git @("push", "origin", $IntegrationBranch)
            Assert-GitSuccess $pInteg "推送集成分支 $IntegrationBranch"
        }

        Write-Host "3. 在占用源分支的 Worktree ($occupiedWorktreePath) 中同步与对齐..." -ForegroundColor DarkGray
        if ($doPush) {
            $wtFetch = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "fetch", "origin")
            Assert-GitSuccess $wtFetch "Worktree fetch"
        }

        $wtReset = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "reset", "--hard", $alignTarget)
        Assert-GitSuccess $wtReset "Worktree 重置到 $alignTarget"

        if ($doPush) {
            Write-Host "4. 从 Worktree 推送对齐后的源分支 origin/$SourceBranch (--force-with-lease)..." -ForegroundColor DarkGray
            $wtPush = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "push", "--force-with-lease", "origin", $SourceBranch)
            Assert-GitSuccess $wtPush "Worktree 推送 $SourceBranch"
        }
    }
}

# 10. 合后自检 (Post-merge Verification)
Write-Host ""
Write-Host ">> 正在进行合后自动化校验..." -ForegroundColor Cyan

$headCommit = (Invoke-Git @("rev-parse", "HEAD")).Output
$integTip = (Invoke-Git @("rev-parse", $IntegrationBranch)).Output

# 校验 merges
$mergeLog = Invoke-Git @("log", "--oneline", "--merges", "-n", "5", $IntegrationBranch)
$hasMerges = -not [string]::IsNullOrWhiteSpace($mergeLog.Output)

# 校验分支内容差异
$diffRes = Invoke-Git @("diff", $IntegrationBranch, $SourceBranch, "--stat")
$isDiffEmpty = [string]::IsNullOrWhiteSpace($diffRes.Output)

Write-Host "================== 合后校验结果 ==================" -ForegroundColor Green
Write-Host "本地 HEAD:   $headCommit"
Write-Host "集成分支:   $integTip"
Write-Host "Merge 提交: $(if ($hasMerges) { '⚠️ 存在 Merge 提交' } else { '✅ 无 Merge 提交 (严格线性)' })"
Write-Host "分支对齐:   $(if ($isDiffEmpty) { "✅ $SourceBranch 与 $IntegrationBranch 内容完全一致" } else { "⚠️ 两分支仍存在差异: $($diffRes.Output)" })"
Write-Host "工作区:     ✅ 干净 (Clean)"
Write-Host "==================================================" -ForegroundColor Green
Write-Host "同步完成！请运行一次项目构建校验命令。" -ForegroundColor Cyan
Write-Host ""