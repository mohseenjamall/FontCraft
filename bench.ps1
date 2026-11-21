# Speed test: FontBase (original) vs FontCraft (this app).
# Usage: powershell -ExecutionPolicy Bypass -File bench.ps1 [-Runs 3] [-SettleSeconds 20]
# Stops FontBase while testing (if it was running) and starts it again at the end.
param(
    [int]$Runs = 3,
    [int]$SettleSeconds = 20,
    [string]$FontBase = "$env:LOCALAPPDATA\Programs\FontBase\FontBase.exe",
    [string]$FontCraft = "$PSScriptRoot\target\release\fontcraft.exe"
)
$ErrorActionPreference = 'Stop'

function Get-Tree([string]$name) { @(Get-Process -Name $name -ErrorAction SilentlyContinue) }

function Stop-App([string]$name) {
    Get-Tree $name | ForEach-Object { try { $_.Kill() } catch {} }
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ((Get-Tree $name).Count -and $sw.Elapsed.TotalSeconds -lt 15) { Start-Sleep -Milliseconds 200 }
}

# One launch: time until a visible, responding main window, then resources after settling.
function Measure-Launch([string]$exe, [string]$name) {
    Stop-App $name
    Start-Sleep -Seconds 2
    $sw = [Diagnostics.Stopwatch]::StartNew()
    Start-Process $exe | Out-Null
    $window = $null; $idle = $null; $show = $null
    $lastCpu = 0.0; $lastT = 0.0; $quiet = 0
    # Idle = all its processes together use under 3% of one core for 1 s straight.
    while ($sw.Elapsed.TotalSeconds -lt 60 -and -not $idle) {
        $tree = Get-Tree $name
        if (-not $window -and ($tree | Where-Object { $_.MainWindowHandle -ne 0 -and $_.Responding })) {
            $window = [math]::Round($sw.Elapsed.TotalSeconds, 2)
        }
        $t = $sw.Elapsed.TotalSeconds
        if ($t - $lastT -ge 0.25) {
            $cpu = ($tree | ForEach-Object { try { $_.TotalProcessorTime.TotalSeconds } catch { 0 } } | Measure-Object -Sum).Sum
            $quiet = if ($t -gt 1 -and ($cpu - $lastCpu) / ($t - $lastT) -lt 0.03) { $quiet + 1 } else { 0 }
            if ($quiet -ge 4) { $idle = [math]::Round($t - 1, 2) }
            $lastCpu = $cpu; $lastT = $t
        }
        Start-Sleep -Milliseconds 25
    }
    if (-not $window) {
        # Started hidden (tray): launching again asks the running copy to open its window.
        $sw2 = [Diagnostics.Stopwatch]::StartNew()
        Start-Process $exe | Out-Null
        while ($sw2.Elapsed.TotalSeconds -lt 20) {
            if (Get-Tree $name | Where-Object { $_.MainWindowHandle -ne 0 -and $_.Responding }) {
                $show = [math]::Round($sw2.Elapsed.TotalSeconds, 2); break
            }
            Start-Sleep -Milliseconds 25
        }
    }
    Start-Sleep -Seconds $SettleSeconds
    $tree = Get-Tree $name
    $result = [pscustomobject]@{
        WindowSeconds = $window
        IdleSeconds   = $idle
        ShowFromTraySeconds = $show
        Processes     = $tree.Count
        WorkingSetMB  = [math]::Round((($tree | Measure-Object WorkingSet64 -Sum).Sum) / 1MB)
        PrivateMB     = [math]::Round((($tree | Measure-Object PrivateMemorySize64 -Sum).Sum) / 1MB)
        CpuSeconds    = [math]::Round((($tree | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum), 2)
        Threads       = ($tree | ForEach-Object { $_.Threads.Count } | Measure-Object -Sum).Sum
    }
    Stop-App $name
    $result
}

# FontCraft only: time until every font is listed and every visible preview is drawn
# (its --screenshot mode waits for that before saving the picture and quitting).
function Measure-FontCraftReady {
    $png = Join-Path $env:TEMP "fontcraft-bench.png"
    Remove-Item $png -ErrorAction SilentlyContinue
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process $FontCraft -ArgumentList '--screenshot', $png -PassThru
    $p.WaitForExit(120000) | Out-Null
    [math]::Round($sw.Elapsed.TotalSeconds, 2)
}

function Get-Median($values) {
    $v = @($values | Where-Object { $_ -ne $null } | Sort-Object)
    if (-not $v.Count) { return $null }
    $v[[int][math]::Floor(($v.Count - 1) / 2)]
}

$wasRunning = (Get-Tree 'FontBase').Count -gt 0
$rows = @()
foreach ($app in @(@{ Label = 'FontBase'; Exe = $FontBase; Name = 'FontBase' },
                   @{ Label = 'FontCraft'; Exe = $FontCraft; Name = 'fontcraft' })) {
    for ($i = 1; $i -le $Runs; $i++) {
        $r = Measure-Launch $app.Exe $app.Name
        $r | Add-Member Label $app.Label
        $r | Add-Member Run $i
        $rows += $r
        Write-Host ("{0} run {1}: window {2}s, idle {3}s, tray-open {4}s, {5} processes, {6} MB working set, {7} MB private, {8}s CPU, {9} threads" -f `
            $app.Label, $i, $r.WindowSeconds, $r.IdleSeconds, $r.ShowFromTraySeconds, $r.Processes, $r.WorkingSetMB, $r.PrivateMB, $r.CpuSeconds, $r.Threads)
    }
}
$ready = 1..$Runs | ForEach-Object { Measure-FontCraftReady }
Write-Host ("FontCraft fonts listed and drawn: {0} s" -f ($ready -join ', '))

$size = @{
    FontBase  = [math]::Round(((Get-ChildItem (Split-Path $FontBase) -Recurse -File | Measure-Object Length -Sum).Sum) / 1MB, 1)
    FontCraft = [math]::Round((Get-Item $FontCraft).Length / 1MB, 1)
}

$summary = foreach ($label in 'FontBase', 'FontCraft') {
    $set = $rows | Where-Object Label -eq $label
    [pscustomobject]@{
        Label         = $label
        WindowSeconds = Get-Median ($set.WindowSeconds)
        IdleSeconds   = Get-Median ($set.IdleSeconds)
        ShowFromTraySeconds = Get-Median ($set.ShowFromTraySeconds)
        Processes     = Get-Median ($set.Processes)
        WorkingSetMB  = Get-Median ($set.WorkingSetMB)
        PrivateMB     = Get-Median ($set.PrivateMB)
        CpuSeconds    = Get-Median ($set.CpuSeconds)
        Threads       = Get-Median ($set.Threads)
        DiskMB        = $size[$label]
    }
}
$out = [pscustomobject]@{ runs = $rows; median = $summary; fontcraft_ready_seconds = $ready; settle_seconds = $SettleSeconds }
$out | ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 (Join-Path $PSScriptRoot 'bench-results.json')
$summary | Format-Table -AutoSize

if ($wasRunning) { Start-Process $FontBase | Out-Null; Write-Host 'FontBase started again.' }
