param(
    [Parameter(Mandatory = $true)][string]$Control,
    [string]$Socket = "$env:APPDATA\horizon-launcher\game-control.json",
    [string]$InstanceHome = "$env:APPDATA\horizon\instances\nocubes-port-validation",
    [int]$Seconds = 10,
    [int]$Distance = 8,
    [int]$MaxFps = 260,
    [string]$Output = "$PSScriptRoot\..\validation\matched-runtime.json"
)
$ErrorActionPreference = 'Stop'
function Invoke-Control {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)
    $response = & $Control --socket $Socket @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Control failed: $Arguments $response" }
    $value = ($response -join "`n") | ConvertFrom-Json
    if ($value.error) { throw "Control failed: $Arguments $($value.error)" }
    return $value
}
function Set-Mode {
    param([bool]$Enabled)
    $config = if ($Enabled) { '{}' } else { '{"render":false,"collisions":false,"extend_fluids_range":0}' }
    [IO.File]::WriteAllText($configPath, $config)
    Invoke-Control -Arguments @('wait', '60') | Out-Null
    # wait-ready alone can report ready before the one-second config reload.
    Invoke-Control -Arguments @('wait-ready', '60') | Out-Null
}
$configPath = Join-Path $InstanceHome 'config\nocubes.json'
$configPresent = Test-Path -LiteralPath $configPath
$original = if ($configPresent) { [IO.File]::ReadAllBytes($configPath) } else { $null }
[IO.Directory]::CreateDirectory((Split-Path -Parent $configPath)) | Out-Null
$samples = @()
try {
    Invoke-Control -Arguments @('options-set', 'renderDistance', "$Distance") | Out-Null
    Invoke-Control -Arguments @('options-set', 'simulationDistance', '8') | Out-Null
    Invoke-Control -Arguments @('options-set', 'enableVsync', 'false') | Out-Null
    Invoke-Control -Arguments @('options-set', 'maxFps', "$MaxFps") | Out-Null
    Invoke-Control -Arguments @('options-set', 'pauseOnLostFocus', 'false') | Out-Null
    Invoke-Control -Arguments @('options-set', 'inactivityFpsLimit', 'minimized') | Out-Null
    Invoke-Control -Arguments @('command', 'gamemode survival') | Out-Null
    Invoke-Control -Arguments @('tp', '-25.5', '82', '5.5') | Out-Null
    Invoke-Control -Arguments @('look', '0', '35') | Out-Null
    foreach ($enabled in @($false, $true, $true, $false)) {
        Set-Mode $enabled
        $state = Invoke-Control -Arguments @('state')
        $status = Invoke-Control -Arguments @('status')
        $stationary = Invoke-Control -Arguments @('bench', "$Seconds")
        Invoke-Control -Arguments @('stats', 'reset') | Out-Null
        for ($lap = 0; $lap -lt 4; $lap++) {
            Invoke-Control -Arguments @('move', 'forward', '0.6') | Out-Null
            Invoke-Control -Arguments @('move', 'back', '0.6') | Out-Null
        }
        $walking = Invoke-Control -Arguments @('stats')
        $samples += [ordered]@{
            enabled = $enabled; state = $state; status = $status
            stationary = $stationary; walking = $walking
        }
        Invoke-Control -Arguments @('tp', '-25.5', '82', '5.5') | Out-Null
        $report = [ordered]@{
            baseline = 'Horizon vanilla rendering/collision paths in the same executable; NoCubes disabled by config, adapter hooks remain installed'
            distance = $Distance; max_fps_setting = $MaxFps; samples = $samples
        } | ConvertTo-Json -Depth 40
        [IO.File]::WriteAllText($Output, $report)
        Write-Output ("enabled={0}: stationary={1:N1} FPS, walking={2:N1} FPS" -f $enabled, $stationary.avg_fps, $walking.avg_fps)
    }
} finally {
    if ($configPresent) { [IO.File]::WriteAllBytes($configPath, $original) }
    else { Remove-Item -LiteralPath $configPath -ErrorAction SilentlyContinue }
}
