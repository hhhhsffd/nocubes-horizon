param(
    [Parameter(Mandatory = $true)][string]$Control,
    [string]$Socket = "$env:APPDATA\horizon-launcher\game-control.json",
    [string]$InstanceHome = "$env:APPDATA\horizon\instances\nocubes-port-validation"
)
$ErrorActionPreference = 'Stop'
function Ctl {
    param([string[]]$Arguments)
    $reply = & $Control --socket $Socket @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Control failed: $Arguments $reply" }
    $value = ($reply -join "`n") | ConvertFrom-Json
    if ($value.error) { throw "Control failed: $Arguments $($value.error)" }
    $value
}
function Ready-Lod {
    $deadline = [DateTime]::UtcNow.AddSeconds(90)
    $stable = 0
    while ([DateTime]::UtcNow -lt $deadline) {
        Ctl -Arguments @('wait', '20') | Out-Null
        $status = Ctl -Arguments @('status')
        $lod = $status.lod
        if ($lod.target_tiles -gt 0 -and $lod.sampled_target_tiles -ge $lod.target_tiles -and
            $lod.resident_quads -gt 0 -and $status.pending_chunks -eq 0) {
            # LOD detail and mesh fades keep changing the resident count even
            # after sampling completes. Check ready inputs for several frames;
            # the on/off geometry assertions below check the visual change.
            $stable++
            if ($stable -ge 3) { return $status }
        } else { $stable = 0 }
    }
    throw 'LOD sampling/meshing did not settle within 90 seconds'
}
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$configPath = Join-Path $InstanceHome 'config\nocubes.json'
$original = [IO.File]::ReadAllBytes($configPath)
$initial = Ctl -Arguments @('state')
if ($initial.world -ne 'NoCubesValidation') { throw 'Open the isolated NoCubesValidation world first' }
$originalStatus = Ctl -Arguments @('status')
$result = [ordered]@{ version = '0.2.4'; modes = @(); visual_review_required = $true }
try {
    Ctl -Arguments @('options-set', 'renderDistance', '4') | Out-Null
    Ctl -Arguments @('lod-distance', '64') | Out-Null
    Ctl -Arguments @('command', 'gamemode spectator') | Out-Null
    Ctl -Arguments @('tp', '-80', '135', '-80') | Out-Null
    Ctl -Arguments @('look', '-35', '15') | Out-Null
    foreach ($mode in @('enabled', 'disabled', 'reenabled')) {
        if ($mode -eq 'disabled') { [IO.File]::WriteAllText($configPath, '{"render":false}') }
        else { [IO.File]::WriteAllBytes($configPath, $original) }
        # Allow the one-second file reload before checking LOD queues.
        Ctl -Arguments @('wait', '160') | Out-Null
        $status = Ready-Lod
        if ($status.lod.active_tiles -le 0 -or @($status.lod.levels).Count -lt 2) {
            throw 'Expected active LOD tiles at several distance levels'
        }
        Ctl -Arguments @('screenshot', (Join-Path $root "validation\screenshots\lod-$mode.png")) | Out-Null
        $result.modes += @{ mode = $mode; status = $status }
        Write-Output "${mode}: $($status.lod.active_tiles) LOD tiles, $($status.lod.resident_quads) LOD quads, $($status.chunks) loaded chunks"
    }
    $enabledQuads = $result.modes[0].status.lod.resident_quads
    $disabledQuads = $result.modes[1].status.lod.resident_quads
    $reenabledQuads = $result.modes[2].status.lod.resident_quads
    if ([Math]::Abs($enabledQuads - $disabledQuads) -lt $enabledQuads / 10 -or
        [Math]::Abs($reenabledQuads - $disabledQuads) -lt $reenabledQuads / 10) {
        throw 'Non-flat validation terrain did not change its LOD geometry when smoothing was disabled'
    }
    Ctl -Arguments @('lod-distance', '256') | Out-Null
    $wide = Ready-Lod
    $result.wide = $wide
    Ctl -Arguments @('screenshot', (Join-Path $root 'validation\screenshots\lod-wide.png')) | Out-Null
    Write-Output "Wide LOD: $($wide.lod.active_tiles) tiles at $(@($wide.lod.levels).Count) levels"
    $result.runtime_passed = $true
} finally {
    [IO.File]::WriteAllBytes($configPath, $original)
    Ctl -Arguments @('options-set', 'renderDistance', "$($originalStatus.render_distance)") | Out-Null
    Ctl -Arguments @('lod-distance', "$($originalStatus.lod.distance)") | Out-Null
    [IO.File]::WriteAllText((Join-Path $root 'validation\lod-smoke.json'), ($result | ConvertTo-Json -Depth 30))
}
