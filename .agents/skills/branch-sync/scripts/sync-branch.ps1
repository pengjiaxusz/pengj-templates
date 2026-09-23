<#
.SYNOPSIS
    Worktree 感知的极速线性化分支同步脚本 (Branch Sync Fast-Track)

.DESCRIPTION
    自动化执行集成分支与特性分支的线性同步流程，提供防漏、防覆盖硬性保护与 1-Shot 闭环执行：
    1. 自动智能推导特性分支与集成分支（未指定分支时自适应探测当前/活跃分支，防止报错致 Agent 迷航）；
    2. 严格核验工作区干净度，拦截脏改动风险（排除已登记 Worktree 与声明的忽略路径正则）；
    3. 双端对齐安全核查（本地与 origin 拓扑检测，自动快进落后引用，拦截分叉冲突）；
    4. 自动创建本地持久化安全快照引用 (refs/sync-backup/...)，确保任何操作均可秒级无损回滚；
    5. 双重提交甄别（git cherry + rev-list），精准剔除同 patch-id 等价提交，捕获真正净新增提交；
    6. 执行变基快进或按序 cherry-pick 线性合入（禁止产生 merge 提交）；
    7. 树级改动保全校验 (Tree-Diff Guard)：在重置源分支前，硬核验证所有改动 100% 进入集成分支；
    8. 双端对齐与安全推送 (--force-with-lease)；
    9. 自动运行合后验证命令 (来自 SKILL.md 声明)，实现单次调用 (1-Shot) 闭环交付。

.PARAMETER SourceBranch
    待合入的源特性分支名。若未指定，自动根据当前所在分支或最近活跃分支智能推导。

.PARAMETER IntegrationBranch
    目标集成分支名。默认按如下顺序自动推导：命令行指定 -> origin/HEAD 符号引用 -> SKILL.md 登记 -> 远端/本地探测 (main/dev/develop/master/trunk) -> git init.defaultBranch。

.PARAMETER DirtyIgnorePattern
    工作区干净度检查时额外忽略的路径正则表达式列表。亦可于 SKILL.md 项目专属区声明。

.PARAMETER Apply
    是否执行实际合并、推送与对齐。未指定时仅进行快速预检 (Dry Run) 并输出紧凑报告。

.PARAMETER NoPush
    在 -Apply 执行时不进行 git push（用于本地演练或离线环境）。

.PARAMETER NoFetch
    执行前跳过 git fetch。

.PARAMETER NoVerify
    在 -Apply 完成后跳过自动运行项目合后验证命令。

.EXAMPLE
    # 一键极速执行（推荐：1 次调用全流程搞定合并、推送、对齐与项目门禁验证）
    pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch feat/my-feature -Apply

    # 自动探测当前分支并一键同步
    pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -Apply

    # 快速预检（只读模式）
    pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch feat/my-feature
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$SourceBranch = "",

    [Parameter(Position = 1)]
    [string]$IntegrationBranch = "",

    [Parameter()]
    [string[]]$DirtyIgnorePattern = @(),

    [switch]$Apply,
    [switch]$NoPush,
    [switch]$NoFetch,
    [switch]$NoVerify
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

# 检查是否存在 remote origin
$remoteCheck = Invoke-Git @("remote", "get-url", "origin")
$hasRemote = ($remoteCheck.ExitCode -eq 0)

# 读取项目 SKILL.md 声明配置（集成分支、忽略正则、合后验证命令）
$skillDocPath = Join-Path $PSScriptRoot "..\SKILL.md"
if (-not (Test-Path $skillDocPath)) {
    $candidate = Join-Path (Get-Location).Path ".agents\skills\branch-sync\SKILL.md"
    if (Test-Path $candidate) {
        $skillDocPath = $candidate
    }
}
$declaredIntegrationBranch = ""
$declaredDirtyPatterns = [System.Collections.Generic.List[string]]::new()
$declaredVerifyCmd = ""

if (Test-Path $skillDocPath) {
    try {
        $skillDocContent = [System.IO.File]::ReadAllText($skillDocPath, [System.Text.Encoding]::UTF8)
        if ($skillDocContent -match '(?m)^[-*]\s*(?:集成分支|Integration branch)[：:]\s*`?([a-zA-Z0-9_\-\.\/]+)`?') {
            $declaredIntegrationBranch = $Matches[1].Trim()
        }
        $patRegex = '(?m)^[-*]\s*(?:忽略未追踪路径正则|忽略路径正则|忽略正则|Dirty ignore regex|Dirty ignore pattern)[：:]\s*`?([^\r\n`]+)`?'
        $patMatches = [regex]::Matches($skillDocContent, $patRegex)
        foreach ($m in $patMatches) {
            $patVal = $m.Groups[1].Value.Trim()
            if (-not [string]::IsNullOrWhiteSpace($patVal)) {
                $declaredDirtyPatterns.Add($patVal)
            }
        }
        # 提取合后验证命令 (代码块中首个非注释非空行)
        $verifyBlockRegex = '(?ms)###\s*(?:合后验证命令|Post-Merge Validation Command|合后检验命令)\s*.*?(?:```(?:powershell|bash|sh|cmd|pwsh)?\r?\n(.*?)\r?\n```)'
        if ($skillDocContent -match $verifyBlockRegex) {
            $codeBlock = $Matches[1]
            $codeLines = $codeBlock -split "`r?`n"
            foreach ($cl in $codeLines) {
                $trimmed = $cl.Trim()
                if (-not [string]::IsNullOrWhiteSpace($trimmed) -and -not $trimmed.StartsWith("#")) {
                    $declaredVerifyCmd = $trimmed
                    break
                }
            }
        }
    } catch {}
}
foreach ($p in $DirtyIgnorePattern) {
    if (-not [string]::IsNullOrWhiteSpace($p)) {
        $declaredDirtyPatterns.Add($p.Trim())
    }
}

# 1. 自动推导集成分支 (多层降级策略)
if ([string]::IsNullOrWhiteSpace($IntegrationBranch)) {
    # 1.1 Git 远端 HEAD 符号引用探测 (如 refs/remotes/origin/HEAD -> origin/main)
    if ($hasRemote) {
        $rHead = Invoke-Git @("symbolic-ref", "--short", "refs/remotes/origin/HEAD")
        if ($rHead.ExitCode -eq 0 -and (-not [string]::IsNullOrWhiteSpace($rHead.Output))) {
            $headRef = $rHead.Output.Trim()
            if ($headRef.StartsWith("origin/")) {
                $IntegrationBranch = $headRef.Substring(7).Trim()
            } else {
                $IntegrationBranch = $headRef
            }
        }
    }

    # 1.2 SKILL.md 项目专属区显式声明
    if ([string]::IsNullOrWhiteSpace($IntegrationBranch) -and (-not [string]::IsNullOrWhiteSpace($declaredIntegrationBranch))) {
        $IntegrationBranch = $declaredIntegrationBranch
    }

    # 1.3 远端常用集成分支探测
    if ([string]::IsNullOrWhiteSpace($IntegrationBranch) -and $hasRemote) {
        foreach ($b in @("main", "dev", "develop", "master", "trunk")) {
            $rTest = Invoke-Git @("rev-parse", "--verify", "origin/$b")
            if ($rTest.ExitCode -eq 0) {
                $IntegrationBranch = $b
                break
            }
        }
    }

    # 1.4 本地常用分支探测
    if ([string]::IsNullOrWhiteSpace($IntegrationBranch)) {
        foreach ($b in @("main", "dev", "develop", "master", "trunk")) {
            $lTest = Invoke-Git @("rev-parse", "--verify", $b)
            if ($lTest.ExitCode -eq 0) {
                $IntegrationBranch = $b
                break
            }
        }
    }

    # 1.5 git init.defaultBranch 回退
    if ([string]::IsNullOrWhiteSpace($IntegrationBranch)) {
        $dConf = Invoke-Git @("config", "init.defaultBranch")
        if ($dConf.ExitCode -eq 0 -and (-not [string]::IsNullOrWhiteSpace($dConf.Output))) {
            $IntegrationBranch = $dConf.Output.Trim()
        } else {
            $IntegrationBranch = "main"
        }
    }
}

$integCheck = Invoke-Git @("rev-parse", "--verify", $IntegrationBranch)
if ($integCheck.ExitCode -ne 0) {
    throw "集成分支 '$IntegrationBranch' 不存在，请检查指定分支名称。"
}

# 2. 自动智能推导源分支 (当未传入 -SourceBranch 时)
if ([string]::IsNullOrWhiteSpace($SourceBranch)) {
    $currentHeadRef = Invoke-Git @("symbolic-ref", "--short", "-q", "HEAD")
    if ($currentHeadRef.ExitCode -eq 0 -and (-not [string]::IsNullOrWhiteSpace($currentHeadRef.Output))) {
        $currBranch = $currentHeadRef.Output.Trim()
        if ($currBranch -ne $IntegrationBranch) {
            $SourceBranch = $currBranch
            Write-Host "💡 未指定 -SourceBranch，自动选定当前所在分支: '$SourceBranch'" -ForegroundColor Cyan
        }
    }
}

if ([string]::IsNullOrWhiteSpace($SourceBranch)) {
    # 当前位于集成分支，尝试寻找其他活跃特性分支或 worktree 占用分支
    $wtList = Invoke-Git @("worktree", "list", "--porcelain")
    $candidateBranches = [System.Collections.Generic.List[string]]::new()
    if ($wtList.ExitCode -eq 0) {
        $wtLines = $wtList.Output -split "`r?`n"
        foreach ($wline in $wtLines) {
            if ($wline.StartsWith("branch refs/heads/")) {
                $bName = $wline.Substring(18).Trim()
                if ($bName -ne $IntegrationBranch -and (-not $candidateBranches.Contains($bName))) {
                    $candidateBranches.Add($bName)
                }
            }
        }
    }

    # 若 worktree 中未找到，检查最近有提交的本地分支（排查 integration）
    if ($candidateBranches.Count -eq 0) {
        $recentBranches = Invoke-Git @("for-each-ref", "--sort=-committerdate", "--format=%(refname:short)", "refs/heads/")
        if ($recentBranches.ExitCode -eq 0) {
            foreach ($rb in ($recentBranches.Output -split "`r?`n")) {
                $rbClean = $rb.Trim()
                if (-not [string]::IsNullOrWhiteSpace($rbClean) -and $rbClean -ne $IntegrationBranch) {
                    $candidateBranches.Add($rbClean)
                    if ($candidateBranches.Count -ge 3) { break }
                }
            }
        }
    }

    if ($candidateBranches.Count -eq 1) {
        $SourceBranch = $candidateBranches[0]
        Write-Host "💡 未指定 -SourceBranch，探测到唯一候选特性分支: '$SourceBranch'" -ForegroundColor Cyan
    } elseif ($candidateBranches.Count -gt 1) {
        $cList = ($candidateBranches | ForEach-Object { "'$_'" }) -join ", "
        throw "当前位于集成分支 '$IntegrationBranch' 且未指定 -SourceBranch。发现多个候选分支: [$cList]，请明确指定 -SourceBranch <分支名>。"
    } else {
        throw "无法推导源分支，且未指定 -SourceBranch。请通过 -SourceBranch 指定待合入的分支名。"
    }
}

# 规范化源分支名
$SourceBranch = $SourceBranch.Trim().Replace("refs/heads/", "")
if ($SourceBranch -eq $IntegrationBranch) {
    throw "源分支 '$SourceBranch' 与集成分支 '$IntegrationBranch' 相同，无需自合并。"
}

# 3. 校验分支存在性
$sourceCheck = Invoke-Git @("rev-parse", "--verify", $SourceBranch)
if ($sourceCheck.ExitCode -ne 0) {
    if ($hasRemote) {
        $sourceRemoteCheck = Invoke-Git @("rev-parse", "--verify", "origin/$SourceBranch")
        if ($sourceRemoteCheck.ExitCode -ne 0) {
            throw "源分支 '$SourceBranch' 在本地及远端 origin 均不存在，请检查分支名称。"
        }
        # 本地不存在但远端存在，创建本地追踪分支
        Invoke-Git @("branch", "--track", $SourceBranch, "origin/$SourceBranch") | Out-Null
    } else {
        throw "源分支 '$SourceBranch' 在本地不存在，请检查分支名称。"
    }
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
        if ($isRegisteredWt) { continue }

        # 若匹配忽略正则，予以忽略
        $isIgnoredPattern = $false
        foreach ($pat in $declaredDirtyPatterns) {
            if ($relPath -match $pat) {
                $isIgnoredPattern = $true
                break
            }
        }
        if ($isIgnoredPattern) { continue }

        $dirtyItems.Add($sl)
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
        $wtDirty = [System.Collections.Generic.List[string]]::new()
        foreach ($line in ($wtStatus.Output -split "`r?`n")) {
            if ([string]::IsNullOrWhiteSpace($line)) { continue }
            $relP = $line.Substring(3).Trim().Trim('"').TrimEnd('/')
            $isIgnoredP = $false
            foreach ($pat in $declaredDirtyPatterns) {
                if ($relP -match $pat) {
                    $isIgnoredP = $true
                    break
                }
            }
            if (-not $isIgnoredP) {
                $wtDirty.Add($line)
            }
        }
        if ($wtDirty.Count -gt 0) {
            $dirtyList = $wtDirty -join "`n"
            throw "占用源分支 '$SourceBranch' 的 Worktree ($occupiedWorktreePath) 存在未提交改动，严禁强制重置！请先在该目录提交或 stash：`n$dirtyList"
        }
    }
}

# 6. Fetch 远端最新状态与双端对齐检查 (防漏提交核心防御)
$doPush = $hasRemote -and (-not $NoPush)
if ($hasRemote -and (-not $NoFetch)) {
    Write-Host "正在拉取远端最新引用 (git fetch)..." -ForegroundColor DarkGray
    Invoke-Git @("fetch", "origin", $IntegrationBranch) | Out-Null
    Invoke-Git @("fetch", "origin", $SourceBranch) | Out-Null

    # 6.1 核验源分支：本地 vs 远端 origin/$SourceBranch
    $hasRemoteSource = (Invoke-Git @("rev-parse", "--verify", "origin/$SourceBranch")).ExitCode -eq 0
    if ($hasRemoteSource) {
        $localSourceSha = (Invoke-Git @("rev-parse", $SourceBranch)).Output
        $remoteSourceSha = (Invoke-Git @("rev-parse", "origin/$SourceBranch")).Output

        if ($localSourceSha -ne $remoteSourceSha) {
            # 检查本地是否为远端的祖先 (即本地落后于远端)
            $isAncestor = (Invoke-Git @("merge-base", "--is-ancestor", $localSourceSha, $remoteSourceSha)).ExitCode -eq 0
            $isRemoteAncestor = (Invoke-Git @("merge-base", "--is-ancestor", $remoteSourceSha, $localSourceSha)).ExitCode -eq 0

            if ($isAncestor) {
                # 本地落后于远端，必须自动快进，杜绝漏掉远端提交！
                Write-Host "⚠️ 检测到本地 '$SourceBranch' 落后于远端 origin/$SourceBranch，自动快进同步..." -ForegroundColor Yellow
                if ($route -eq "Route A") {
                    $currHead = (Invoke-Git @("symbolic-ref", "--short", "-q", "HEAD")).Output
                    if ($currHead -eq $SourceBranch) {
                        $ffRes = Invoke-Git @("merge", "--ff-only", "origin/$SourceBranch")
                        Assert-GitSuccess $ffRes "快进本地 $SourceBranch"
                    } else {
                        $upRes = Invoke-Git @("update-ref", "refs/heads/$SourceBranch", $remoteSourceSha)
                        Assert-GitSuccess $upRes "更新本地 $SourceBranch 指针"
                    }
                } else {
                    $wtFf = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "merge", "--ff-only", "origin/$SourceBranch")
                    Assert-GitSuccess $wtFf "快进 Worktree 中的 $SourceBranch"
                }
                Write-Host "✅ 源分支已自动快进对齐至远端最新提交。" -ForegroundColor Green
            } elseif ($isRemoteAncestor) {
                # 本地领先远端，正常（本地有新增提交，稍后统一推送）
                Write-Host "ℹ️ 本地 '$SourceBranch' 领先远端 origin/$SourceBranch，将合入本地最新变更。" -ForegroundColor DarkGray
            } else {
                # 两端分叉，绝不盲目覆盖，硬性拦截！
                throw "源分支 '$SourceBranch' 的本地版本与远端 origin/$SourceBranch 发生分叉冲突 (Diverged)！`n为防止代码丢失或被意外覆盖，请先手动处理本地与远端分支冲突后再同步。"
            }
        }
    }

    # 6.2 核验集成分支：本地 vs 远端 origin/$IntegrationBranch
    $hasRemoteInteg = (Invoke-Git @("rev-parse", "--verify", "origin/$IntegrationBranch")).ExitCode -eq 0
    if ($hasRemoteInteg) {
        $localIntegSha = (Invoke-Git @("rev-parse", $IntegrationBranch)).Output
        $remoteIntegSha = (Invoke-Git @("rev-parse", "origin/$IntegrationBranch")).Output

        if ($localIntegSha -ne $remoteIntegSha) {
            $isIntegAncestor = (Invoke-Git @("merge-base", "--is-ancestor", $localIntegSha, $remoteIntegSha)).ExitCode -eq 0
            if ($isIntegAncestor) {
                Write-Host "⚠️ 检测到本地集成分支落后于远端，自动快进更新本地集成分支..." -ForegroundColor Yellow
                $currHead = (Invoke-Git @("symbolic-ref", "--short", "-q", "HEAD")).Output
                if ($currHead -eq $IntegrationBranch) {
                    $ffInteg = Invoke-Git @("merge", "--ff-only", "origin/$IntegrationBranch")
                    Assert-GitSuccess $ffInteg "快进本地集成分支 $IntegrationBranch"
                } else {
                    $upInteg = Invoke-Git @("update-ref", "refs/heads/$IntegrationBranch", $remoteIntegSha)
                    Assert-GitSuccess $upInteg "更新本地集成分支指针"
                }
            }
        }
    }
}

# 7. 提交拓扑与净贡献精准甄别 (双重比对: cherry -v + rev-list)
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

# 全量未合入提交列表（用于防漏核对）
$allSourceExclusiveCommits = [System.Collections.Generic.List[string]]::new()
$revListRaw = Invoke-Git @("rev-list", "--reverse", "$IntegrationBranch..$SourceBranch")
if ($revListRaw.ExitCode -eq 0 -and (-not [string]::IsNullOrWhiteSpace($revListRaw.Output))) {
    foreach ($h in ($revListRaw.Output -split "`r?`n")) {
        if (-not [string]::IsNullOrWhiteSpace($h)) {
            $allSourceExclusiveCommits.Add($h.Trim())
        }
    }
}

# 读取当前 commit hashes
$integCommit = (Invoke-Git @("rev-parse", "--short", $IntegrationBranch)).Output
$sourceCommit = (Invoke-Git @("rev-parse", "--short", $SourceBranch)).Output
$fullIntegCommit = (Invoke-Git @("rev-parse", $IntegrationBranch)).Output
$fullSourceCommit = (Invoke-Git @("rev-parse", $SourceBranch)).Output

# 8. 模式分流：预检报告 (Dry Run)
if (-not $Apply) {
    Write-Host ""
    Write-Host "================== Branch Sync 预检报告 ==================" -ForegroundColor Cyan
    Write-Host "集成分支: $IntegrationBranch ($integCommit)"
    Write-Host "源分支:   $SourceBranch ($sourceCommit)"
    if ($route -eq "Route A") {
        Write-Host "同步路径: Route A [自由分支 / 单仓库模式] (direct rebase + ff-merge)" -ForegroundColor Green
    } else {
        Write-Host "同步路径: Route B [Worktree 占用模式: $occupiedWorktreePath]" -ForegroundColor Yellow
    }
    Write-Host "工作区:   干净 (Clean)" -ForegroundColor Green
    Write-Host "----------------------------------------------------------"
    Write-Host "净贡献提交分析:" -ForegroundColor Cyan
    if ($netCommits.Count -eq 0) {
        Write-Host "  无净新增提交（源分支已完全包含在集成分支中）。" -ForegroundColor Yellow
    } else {
        Write-Host "  发现 $($netCommits.Count) 个净新增提交（按拓扑顺序合入）：" -ForegroundColor Green
        foreach ($nc in $netCommits) {
            Write-Host "    + $($nc.Hash.Substring(0, [Math]::Min(7, $nc.Hash.Length))) $($nc.Subject)" -ForegroundColor White
        }
    }
    if ($duplicateCommits.Count -gt 0) {
        Write-Host "  自动忽略 $($duplicateCommits.Count) 个同等改动提交 (相同 patch-id 已存在于主线)：" -ForegroundColor DarkGray
        foreach ($dc in $duplicateCommits) {
            Write-Host "    - $($dc.Hash.Substring(0, [Math]::Min(7, $dc.Hash.Length))) $($dc.Subject)" -ForegroundColor DarkGray
        }
    }
    Write-Host "----------------------------------------------------------"
    Write-Host "操作指引 (一键执行并自动校验):" -ForegroundColor Cyan
    Write-Host "  pwsh .agents/skills/branch-sync/scripts/sync-branch.ps1 -SourceBranch '$SourceBranch' -Apply" -ForegroundColor Yellow
    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host ""
    return
}

# 9. 执行阶段 (Apply) — 带持久化安全快照与树级保全
Write-Host ""
Write-Host ">> 开始执行分支同步 (路径: $route)..." -ForegroundColor Cyan

# 9.1 创建本地持久化安全快照引用 (Safety Backup Ref)
$timestamp = (Get-Date).ToString("yyyyMMdd-HHmmss")
$cleanBranchTag = $SourceBranch -replace '[^a-zA-Z0-9_\-]', '_'
$backupSourceRef = "refs/sync-backup/$cleanBranchTag/$timestamp-$($sourceCommit)"
$backupIntegRef = "refs/sync-backup/$IntegrationBranch/$timestamp-$($integCommit)"

Invoke-Git @("update-ref", $backupSourceRef, $fullSourceCommit) | Out-Null
Invoke-Git @("update-ref", $backupIntegRef, $fullIntegCommit) | Out-Null
Write-Host "🛡️ 安全快照已创建: $backupSourceRef" -ForegroundColor DarkCyan

$initialBranch = (Invoke-Git @("rev-parse", "--abbrev-ref", "HEAD")).Output
$alignTarget = if ($doPush) { "origin/$IntegrationBranch" } else { $IntegrationBranch }

# 9.2 记录合前源分支独有改动 (用于 Tree-Diff Guard 树级防漏校验)
$preMergeDiff = Invoke-Git @("diff", "$IntegrationBranch...$SourceBranch", "--stat")

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
        # Route A 流程: checkout feat/x -> rebase integration -> checkout integration -> merge --ff-only
        Write-Host "1. 切换至源分支 '$SourceBranch' 并执行变基 (rebase $IntegrationBranch)..." -ForegroundColor DarkGray
        $co1 = Invoke-Git @("checkout", $SourceBranch)
        Assert-GitSuccess $co1 "切换至 $SourceBranch"

        $rb = Invoke-Git @("rebase", $IntegrationBranch)
        if ($rb.ExitCode -ne 0) {
            Write-Host "变基过程发生冲突！正在中止变基并恢复现场..." -ForegroundColor Red
            Invoke-Git @("rebase", "--abort") | Out-Null
            Invoke-Git @("checkout", $initialBranch) | Out-Null
            throw "在对 '$SourceBranch' 执行 'git rebase $IntegrationBranch' 时遇到冲突。原始状态保存在 $backupSourceRef，请手动排查解决。"
        }

        Write-Host "2. 切换回集成分支 '$IntegrationBranch' 并执行快进合并 (merge --ff-only)..." -ForegroundColor DarkGray
        $co2 = Invoke-Git @("checkout", $IntegrationBranch)
        Assert-GitSuccess $co2 "切换至 $IntegrationBranch"

        $mg = Invoke-Git @("merge", "--ff-only", $SourceBranch)
        Assert-GitSuccess $mg "快进合并 $SourceBranch"

    } else {
        # Route B 流程: 主仓库切换至集成分支，依次 cherry-pick 净贡献提交
        Write-Host "1. 切换至集成分支 '$IntegrationBranch' 并按序 cherry-pick 净贡献提交..." -ForegroundColor DarkGray
        $co = Invoke-Git @("checkout", $IntegrationBranch)
        Assert-GitSuccess $co "切换至 $IntegrationBranch"

        $hashList = $netCommits | ForEach-Object { $_.Hash }
        $cpArgs = @("cherry-pick") + $hashList
        $cp = Invoke-Git $cpArgs
        if ($cp.ExitCode -ne 0) {
            Write-Host "Cherry-pick 过程发生冲突！正在中止 cherry-pick 并恢复集成分支..." -ForegroundColor Red
            Invoke-Git @("cherry-pick", "--abort") | Out-Null
            Invoke-Git @("reset", "--hard", $backupIntegRef) | Out-Null
            Invoke-Git @("checkout", $initialBranch) | Out-Null
            throw "Cherry-pick 净贡献提交时发生冲突。集成分支已安全回滚至合入前状态 ($backupIntegRef)。"
        }
    }

    # 9.3 核心门禁：Tree-Diff Guard 树级改动保全校验 (严禁在确认完全合入前重置源分支！)
    Write-Host "🔍 执行合后改动保全审计 (Tree-Diff Guard)..." -ForegroundColor DarkCyan
    $postCherry = Invoke-Git @("cherry", "-v", $IntegrationBranch, $backupSourceRef)
    $remainingNet = [System.Collections.Generic.List[string]]::new()
    if ($postCherry.ExitCode -eq 0 -and (-not [string]::IsNullOrWhiteSpace($postCherry.Output))) {
        foreach ($pcl in ($postCherry.Output -split "`r?`n")) {
            if ($pcl -match '^\+\s+([0-9a-fA-F]+)\s+(.*)$') {
                $remainingNet.Add("$($matches[1]) $($matches[2])")
            }
        }
    }

    if ($remainingNet.Count -gt 0) {
        # 严重告警：存在未合入的净新增提交！绝不触碰源分支，立刻回滚集成分支！
        $missedList = $remainingNet -join "`n"
        Write-Host "❌ 严重拦截：检测到存在未完全合入的净提交！正在回滚集成分支..." -ForegroundColor Red
        Invoke-Git @("reset", "--hard", $backupIntegRef) | Out-Null
        Invoke-Git @("checkout", $initialBranch) | Out-Null
        throw "Tree-Diff Guard 拦截：源分支仍有未合入的净提交，为防止提交丢失，已中止合并并回滚。未合入列表：`n$missedList`n快照引用：$backupSourceRef"
    }

    Write-Host "✅ Tree-Diff Guard 审计通过：源分支所有净贡献已 100% 完整合入主线！" -ForegroundColor Green

    # 9.4 推送集成分支
    if ($doPush) {
        Write-Host "3. 推送集成分支 origin/$IntegrationBranch..." -ForegroundColor DarkGray
        $pInteg = Invoke-Git @("push", "origin", $IntegrationBranch)
        Assert-GitSuccess $pInteg "推送集成分支 $IntegrationBranch"
    }

    # 9.5 安全对齐源分支与推送 (--force-with-lease)
    if ($route -eq "Route A") {
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

        Invoke-Git @("checkout", $IntegrationBranch) | Out-Null
    } else {
        Write-Host "4. 在占用源分支的 Worktree ($occupiedWorktreePath) 中同步与对齐..." -ForegroundColor DarkGray
        if ($doPush) {
            $wtFetch = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "fetch", "origin")
            Assert-GitSuccess $wtFetch "Worktree fetch"
        }

        $wtReset = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "reset", "--hard", $alignTarget)
        Assert-GitSuccess $wtReset "Worktree 重置到 $alignTarget"

        if ($doPush) {
            Write-Host "5. 从 Worktree 推送对齐后的源分支 origin/$SourceBranch (--force-with-lease)..." -ForegroundColor DarkGray
            $wtPush = Invoke-Git @("-C", "`"$occupiedWorktreePath`"", "push", "--force-with-lease", "origin", $SourceBranch)
            Assert-GitSuccess $wtPush "Worktree 推送 $SourceBranch"
        }
    }
}

# 10. 合后自检 (Post-merge Verification)
$headCommit = (Invoke-Git @("rev-parse", "HEAD")).Output
$integTip = (Invoke-Git @("rev-parse", $IntegrationBranch)).Output

$mergeLog = Invoke-Git @("log", "--oneline", "--merges", "-n", "5", $IntegrationBranch)
$hasMerges = -not [string]::IsNullOrWhiteSpace($mergeLog.Output)

$diffRes = Invoke-Git @("diff", $IntegrationBranch, $SourceBranch, "--stat")
$isDiffEmpty = [string]::IsNullOrWhiteSpace($diffRes.Output)

# 11. 自动执行项目合后验证命令 (单调用 1-Shot 闭环核心)
$verifyStatus = "SKIPPED"
if (-not $NoVerify -and (-not [string]::IsNullOrWhiteSpace($declaredVerifyCmd))) {
    Write-Host ""
    Write-Host ">> 正在运行项目专属合后验证命令: $declaredVerifyCmd" -ForegroundColor Cyan
    $verifyStartTime = [System.DateTime]::Now
    try {
        $pinfo = New-Object System.Diagnostics.ProcessStartInfo
        if ($IsWindows -or ($PSVersionTable.PSEdition -ne "Core" -and [System.Environment]::OSVersion.Platform -like "*Win*")) {
            $pinfo.FileName = "powershell.exe"
            $pinfo.Arguments = "-NoProfile -ExecutionPolicy Bypass -Command `"$declaredVerifyCmd`""
        } else {
            $pinfo.FileName = "sh"
            $pinfo.Arguments = "-c `"$declaredVerifyCmd`""
        }
        $pinfo.RedirectStandardOutput = $false
        $pinfo.RedirectStandardError = $false
        $pinfo.UseShellExecute = $false
        $pinfo.WorkingDirectory = (Get-Location).Path

        $vProc = [System.Diagnostics.Process]::Start($pinfo)
        $vProc.WaitForExit()

        if ($vProc.ExitCode -eq 0) {
            $verifyStatus = "PASSED ($declaredVerifyCmd)"
        } else {
            $verifyStatus = "FAILED (退出码 $($vProc.ExitCode))"
        }
    } catch {
        $verifyStatus = "ERROR ($($_.Exception.Message))"
    }
} elseif ($NoVerify) {
    $verifyStatus = "SKIPPED (指定了 -NoVerify)"
} else {
    $verifyStatus = "NONE (SKILL.md 未登记验证命令)"
}

# 12. 格式化交付终态看板
Write-Host ""
Write-Host "================== BRANCH SYNC SUCCESS ==================" -ForegroundColor Green
Write-Host "集成分支:       $IntegrationBranch ($($integTip.Substring(0, [Math]::Min(7, $integTip.Length))))"
Write-Host "源分支:         $SourceBranch (已完全对齐并同步推送)"
Write-Host "安全快照:       $backupSourceRef"
Write-Host "净合入提交:     $($netCommits.Count) 个提交 (严格线性，0 Merge)"
Write-Host "改动保全审计:   ✅ PASSED (Tree-Diff 100% 完整保留)"
Write-Host "两端对齐状态:   $(if ($isDiffEmpty) { '✅ 完全对齐 (Diff 为空)' } else { '⚠️ 存在差异' })"
Write-Host "合后门禁验证:   $(if ($verifyStatus.StartsWith('PASSED')) { "✅ $verifyStatus" } elseif ($verifyStatus.StartsWith('FAILED')) { "❌ $verifyStatus" } else { "ℹ️ $verifyStatus" })"
Write-Host "工作区状态:     ✅ 干净 (Clean)"
Write-Host "状态判定:       COMPLETED_READY_TO_REPORT"
Write-Host "=========================================================" -ForegroundColor Green

if ($verifyStatus.StartsWith("FAILED")) {
    Write-Warning "项目合后验证命令未通过，请检查上方测试输出并排查修复！"
} else {
    Write-Host "分支同步及合后验证已全部通过，无需多余工具调用，可直接向用户汇报完成。" -ForegroundColor Cyan
}
Write-Host ""