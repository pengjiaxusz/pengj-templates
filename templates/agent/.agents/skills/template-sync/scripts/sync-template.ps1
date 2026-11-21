<#
.SYNOPSIS
    Layered template sync, batch update, and selective commit pipeline (Template Sync Fast-Track)

.DESCRIPTION
    Batch wrapper around `pengj-templates-cli update`. The CLI owns the sync itself and the
    closing steps (selective staging, conventional commit, push); this script only adds what a
    batch run needs on top of it:
    1. Sub-second detection of pengj-templates-cli binary (local target build > PATH > fallback cargo run);
    2. Batch project directories (-Projects @(...));
    3. Default dry run mode: rapid diagnosis of alignment and drift states via audit;
    4. Single-shot update on -Apply with -SyncSkills enabled by default, committing and pushing
       through `update --commit --push` so the staging/commit rules live in exactly one place;
    5. Summary table across the batch.

.PARAMETER Projects
    List of target project paths. Supports multiple paths e.g. @("D:\repo1", "D:\repo2"). Defaults to @("."). 

.PARAMETER Apply
    Execute actual update. If omitted, performs read-only pre-flight audit (Dry Run).

.PARAMETER SyncSkills
    Force-align pure skill assets that were locally modified (default $true). Untouched assets
    sync on their own; this flag is the aggressive override.

.PARAMETER Commit
    Kept for compatibility: the selective commit is the DEFAULT on -Apply. Use -NoCommit to disable it.

.PARAMETER Push
    Kept for compatibility: the push is the DEFAULT on -Apply. Use -NoPush to disable it.

.PARAMETER NoCommit
    Skip the automatic selective commit after -Apply (also skips push).

.PARAMETER NoPush
    Skip the automatic push but still create the selective commit.

.PARAMETER CommitMessage
    Custom commit message. If omitted, `update` derives the subject from the target project's
    `.pengj-templates/gate.toml` (scope whitelist + subject language).

.PARAMETER Diff
    Display detailed drift diffs in pre-flight mode.

.EXAMPLE
    # 1. Rapid read-only pre-flight audit of template drifts
    pwsh .agents/skills/template-sync/scripts/sync-template.ps1 -Projects @("D:\pengj\proj-a", "D:\pengj\proj-b")

    # 2. Batch update, selective commit, and push in a single tool call (default behavior)
    pwsh .agents/skills/template-sync/scripts/sync-template.ps1 -Projects @("D:\pengj\proj-a", "D:\pengj\proj-b") -Apply

    # 3. Update without committing (explicit opt-out)
    pwsh .agents/skills/template-sync/scripts/sync-template.ps1 -Projects @("D:\pengj\proj-a") -Apply -NoCommit
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
    [string[]]$Projects = @("."),

    [switch]$Apply,
    [bool]$SyncSkills = $true,
    [switch]$Commit,
    [switch]$Push,
    [switch]$NoCommit,
    [switch]$NoPush,
    [string]$CommitMessage = "",
    [switch]$Diff
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# -Apply commits and pushes the applied project by default; -NoCommit / -NoPush opt out.
# -Commit / -Push remain accepted for compatibility with existing command lines.
$doCommit = -not $NoCommit.IsPresent
$doPush = $doCommit -and (-not $NoPush.IsPresent)

# 1. Discover or resolve optimal pengj-templates-cli execution path
function Find-CliCommand {
    if ($env:PENGJ_TEMPLATES_CLI -and (Test-Path $env:PENGJ_TEMPLATES_CLI)) {
        return @{ Type = "bin"; Cmd = $env:PENGJ_TEMPLATES_CLI }
    }
    $scriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
# Walk up to repository root (detect Cargo.toml)
    $curr = $scriptDir
    for ($i = 0; $i -lt 8; $i++) {
        if (Test-Path (Join-Path $curr "Cargo.toml")) {
            # Both artifacts can exist in development: pick the newest one, so a freshly built
            # debug binary is not shadowed by a stale release binary (and vice versa).
            $built = @(
                (Join-Path $curr "target\release\pengj-templates-cli.exe")
                (Join-Path $curr "target\debug\pengj-templates-cli.exe")
            ) | Where-Object { Test-Path $_ } |
                Sort-Object { (Get-Item $_).LastWriteTimeUtc } -Descending
            if ($built.Count -gt 0) {
                return @{ Type = "bin"; Cmd = $built[0] }
            }
            break
        }
        $parent = Split-Path -Parent $curr
        if (-not $parent -or $parent -eq $curr) { break }
        $curr = $parent
    }
    $pathBin = Get-Command "pengj-templates-cli" -ErrorAction SilentlyContinue
    if ($pathBin) {
        return @{ Type = "bin"; Cmd = $pathBin.Source }
    }
    return @{ Type = "cargo"; Cmd = "cargo" }
}

$cliInfo = Find-CliCommand

function Invoke-Cli {
    param([string[]]$CommandArgs)
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    if ($cliInfo.Type -eq "bin") {
        $psi.FileName = $cliInfo.Cmd
        $psi.Arguments = ($CommandArgs -join " ")
    } else {
        $psi.FileName = "cargo"
        $psi.Arguments = (@("run", "-q", "-p", "pengj-templates-cli", "--") + $CommandArgs) -join " "
    }
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.StandardOutputEncoding = [System.Text.Encoding]::UTF8
    $psi.StandardErrorEncoding = [System.Text.Encoding]::UTF8

    $p = [System.Diagnostics.Process]::Start($psi)
    $stdout = $p.StandardOutput.ReadToEnd()
    $stderr = $p.StandardError.ReadToEnd()
    $p.WaitForExit()

    return [PSCustomObject]@{
        ExitCode = $p.ExitCode
        Output   = $stdout.Trim()
        Error    = $stderr.Trim()
    }
}

$summaryRows = @()
$hasAnyViolations = $false

# The closing steps live in the CLI (`update --commit`). Projects pick up this script with a
# template update, while the binary in PATH is only replaced when the user reinstalls it, so an
# old CLI is a real state: detect it once and degrade loudly instead of failing mid-batch.
$cliSupportsCommit = $true
if ($doCommit) {
    $helpRes = Invoke-Cli -CommandArgs @("update", "--help")
    $cliSupportsCommit = ($helpRes.Output -match "--commit")
    if (-not $cliSupportsCommit) {
        Write-Warning "This pengj-templates-cli does not support 'update --commit'; the sync will run without committing. Upgrade the CLI (see 'pengj-templates-cli check-update') to close the loop again."
    }
}

$targetProjects = @()
foreach ($p in $Projects) {
    if ($p -match ",") {
        $targetProjects += ($p -split ",") | ForEach-Object { $_.Trim() }
    } else {
        $targetProjects += $p.Trim()
    }
}
$targetProjects = $targetProjects | Where-Object { $_ }
if ($targetProjects.Count -eq 0) {
    $targetProjects = @(".")
}

Write-Host ">>> Using CLI invocation mode: $($cliInfo.Type) ($($cliInfo.Cmd))" -ForegroundColor Cyan

foreach ($rawProj in $targetProjects) {
    if (-not (Test-Path $rawProj)) {
        Write-Warning "Project path does not exist, skipping: $rawProj"
        continue
    }
    $projDir = (Resolve-Path $rawProj).Path
    $manifestPath = Join-Path $projDir ".pengj-templates/manifest.json"

    if (-not (Test-Path $manifestPath)) {
        Write-Warning "Project unmanaged (missing .pengj-templates/manifest.json), skipping: $projDir"
        continue
    }

    $projectName = Split-Path -Leaf $projDir

    # ---------------- Mode 1: Pre-Flight Audit (Dry Run) ----------------
    if (-not $Apply) {
        Write-Host "`n>>> [Pre-Flight] Auditing project: $projectName ($projDir)" -ForegroundColor Cyan
        $auditArgs = @("audit", "--dir", "`"$projDir`"", "--json")
        $auditRes = Invoke-Cli -CommandArgs $auditArgs
        if ($auditRes.ExitCode -ne 0 -and -not $auditRes.Output) {
            Write-Error "[$projectName] Audit failed: $($auditRes.Error)"
            continue
        }

        try {
            $auditJson = $auditRes.Output | ConvertFrom-Json
        } catch {
            Write-Error "[$projectName] Failed to parse Audit JSON output: $($auditRes.Output)"
            continue
        }

        $inSync = ($auditJson.items | Where-Object { $_.status -eq "InSync" }).Count
        $customized = ($auditJson.items | Where-Object { $_.status -eq "ProjectCustomized" }).Count
        $upstreamNewer = ($auditJson.items | Where-Object { $_.status -eq "UpstreamNewer" }).Count
        $violations = ($auditJson.items | Where-Object { $_.status -in @("ViolatedManagedBlock", "LocallyModifiedFile") }).Count

        if ($auditJson.has_violations) {
            $hasAnyViolations = $true
            Write-Host "  [Warning] Detected $violations violation drifts!" -ForegroundColor Red
        }

        if ($Diff) {
            $diffArgs = @("audit", "--dir", "`"$projDir`"", "--diff")
            $diffRes = Invoke-Cli -CommandArgs $diffArgs
            Write-Host $diffRes.Output
        }

        $summaryRows += [PSCustomObject]@{
            Project       = $projectName
            Mode          = "DryRun"
            InSync        = $inSync
            Customized    = $customized
            UpstreamNewer = $upstreamNewer
            Violations    = $violations
            Action        = if ($upstreamNewer -gt 0) { "Recommend -Apply" } else { "Up to date" }
        }
        continue
    }

    # ---------------- Mode 2: Execute Update (-Apply) ----------------
    Write-Host "`n>>> [Apply] Synchronizing updates: $projectName ($projDir)" -ForegroundColor Cyan
    $updateArgs = @("update", "--dir", "`"$projDir`"")
    if ($SyncSkills) {
        $updateArgs += "--sync-skills"
    }
    # Staging / commit / push belong to the CLI: `--commit` only stages the paths this run
    # touched plus the manifest, and the subject comes from the project's own gate rules.
    if ($doCommit -and $cliSupportsCommit) {
        $updateArgs += "--commit"
        if ($doPush) {
            $updateArgs += "--push"
        }
        if ($CommitMessage) {
            $updateArgs += "--message"
            $updateArgs += "`"$CommitMessage`""
        }
    }

    $updateRes = Invoke-Cli -CommandArgs $updateArgs
    if ($updateRes.ExitCode -ne 0) {
        Write-Error "[$projectName] Update failed: $($updateRes.Error) $($updateRes.Output)"
        continue
    }

    Write-Host $updateRes.Output

    # Parse the report for the summary table (the counts are informational; git work is done by the CLI)
    $updatedFiles = @()
    $createdFiles = @()
    $conflictedFiles = @()
    $needsReviewFiles = @()

    foreach ($line in ($updateRes.Output -split "`r?`n")) {
        if ($line -match '^\s*\[(?:\u66f4\u65b0|Updated?)\]\s+(.+)$') {
            $updatedFiles += $matches[1].Trim()
        } elseif ($line -match '^\s*\[(?:\u65b0\u589e|Added?)\]\s+(.+)$') {
            $createdFiles += $matches[1].Trim()
        } elseif ($line -match '^\s*\[(?:\u51b2\u7a81|Conflicted?)\]\s+(.+)$') {
            $conflictedFiles += $matches[1].Trim()
        } elseif ($line -match '^\s*\[(?:\u5f85\u590d\u6838|NeedsReview)\]\s+(.+)$') {
            $needsReviewFiles += $matches[1].Trim()
        }
    }

    $commitStatus = "Skipped Commit"
    $pushStatus = "Skipped Push"

    if ($doCommit) {
        if (-not $cliSupportsCommit) {
            $commitStatus = "CLI Too Old"
        } elseif ($updateRes.Output -match '\[(?:\u63d0\u4ea4|Committed?)\]\s+(?!.*\u8df3\u8fc7)') {
            $commitStatus = "Committed"
        } elseif ($updateRes.Output -match '\[(?:\u63d0\u4ea4|Committed?)\]') {
            $commitStatus = "No changes"
            $pushStatus = "No changes"
        } else {
            $commitStatus = "Commit Skipped"
        }
        if ($doPush -and $updateRes.Output -match '\[(?:\u63a8\u9001|Pushed)\]') {
            $pushStatus = "Pushed"
        }
    }

    $summaryRows += [PSCustomObject]@{
        Project      = $projectName
        Mode         = "Apply"
        Updated      = $updatedFiles.Count
        Created      = $createdFiles.Count
        Conflicted   = $conflictedFiles.Count
        NeedsReview  = $needsReviewFiles.Count
        CommitStatus = $commitStatus
        PushStatus   = $pushStatus
    }
}

Write-Host "`n========================= Execution Summary =========================" -ForegroundColor Green
$summaryRows | Format-Table -AutoSize

if ($hasAnyViolations) {
    Write-Host "[Notice] Violation drifts detected; follow standards to resolve or report upstream!`n" -ForegroundColor Yellow
}
