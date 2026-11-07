param(
    [Parameter(Mandatory = $true)][string]$Control,
    [string]$Socket = "$env:APPDATA\horizon-launcher\game-control.json",
    [string]$InstanceHome = "$env:APPDATA\horizon\instances\nocubes-port-validation",
    [string]$Output = "$PSScriptRoot\..\validation\runtime-smoke.json"
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
function Mode {
    param([string]$Mesher)
    [IO.File]::WriteAllText($configPath, ('{"mesher":"' + $Mesher + '"}'))
    Ctl -Arguments @('wait', '60') | Out-Null
    Ctl -Arguments @('wait-ready', '60') | Out-Null
}
function Is-TestBlock {
    param($State)
    ($null -ne $State.looking_at.block) -and
        (($State.looking_at.block.pos -join ',') -eq '-26,82,8')
}
$configPath = Join-Path $InstanceHome 'config\nocubes.json'
$present = Test-Path -LiteralPath $configPath
$original = if ($present) { [IO.File]::ReadAllBytes($configPath) } else { $null }
$oldBlock = Ctl -Arguments @('block', '-26', '82', '8')
$result = [ordered]@{ modes = @() }
try {
    Mode 'SurfaceNets'
    Ctl -Arguments @('command', 'gamemode survival') | Out-Null
    Ctl -Arguments @('tp', '-25.5', '82', '5.5') | Out-Null
    Ctl -Arguments @('wait', '30') | Out-Null
    Ctl -Arguments @('look', '0', '0') | Out-Null
    Ctl -Arguments @('setblock', '-26', '82', '8', 'minecraft:stone') | Out-Null
    Ctl -Arguments @('wait', '10') | Out-Null
    $before = Ctl -Arguments @('state')
    if (!(Is-TestBlock $before) -or !$before.player.on_ground) { throw 'Expected a picked test block and a stationary grounded player' }
    # Warm the same ray query repeatedly before editing its sampled world.
    Ctl -Arguments @('bench', '2') | Out-Null
    Ctl -Arguments @('setblock', '-26', '82', '8', 'minecraft:air') | Out-Null
    Ctl -Arguments @('wait', '5') | Out-Null
    $removed = Ctl -Arguments @('state')
    if (Is-TestBlock $removed) { throw 'Removed block still appears in cached ray geometry' }
    if (($before.player.position -join ',') -ne ($removed.player.position -join ',')) { throw 'Player moved during cache invalidation test' }
    Ctl -Arguments @('setblock', '-26', '82', '8', 'minecraft:stone') | Out-Null
    Ctl -Arguments @('wait', '5') | Out-Null
    $restored = Ctl -Arguments @('state')
    if (!(Is-TestBlock $restored)) { throw 'Restored block missing from cached ray geometry' }
    Mode 'CullingCubic'
    $cubic = Ctl -Arguments @('state')
    if (!(Is-TestBlock $cubic)) { throw 'Cubic mode did not pick the test block' }
    if ([Math]::Abs($before.looking_at.block.point[2] - $cubic.looking_at.block.point[2]) -lt 0.001) {
        throw 'Mesher configuration change did not change the isolated surface'
    }
    $result.cache_invalidation = @{ before = $before; removed = $removed; restored = $restored; cubic = $cubic }
    Ctl -Arguments @('command', 'gamemode spectator') | Out-Null
    Ctl -Arguments @('wait', '5') | Out-Null
    Ctl -Arguments @('tp', '-28.5', '87', '0.5') | Out-Null
    Ctl -Arguments @('look', '-30', '40') | Out-Null
    $screenshots = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\validation\screenshots'))
    [IO.Directory]::CreateDirectory($screenshots) | Out-Null
    foreach ($mesher in @('SurfaceNets', 'SurfaceNets2x', 'MarchingCubes', 'MarchingCubes2x',
            'CullingCubic', 'StupidCubic', 'OldNoCubes', 'CullingChamfer', 'Wulferis')) {
        Mode $mesher
        $state = Ctl -Arguments @('state')
        $status = Ctl -Arguments @('status')
        $result.modes += @{ mesher = $mesher; state = $state; status = $status }
        Ctl -Arguments @('screenshot', (Join-Path $screenshots "$mesher.png")) | Out-Null
        Write-Output "${mesher}: running, $($status.pending_chunks) pending chunks"
    }
    Mode 'SurfaceNets'
    Ctl -Arguments @('command', 'gamemode survival') | Out-Null
    Ctl -Arguments @('tp', '-28.5', '82', '3.5') | Out-Null
    Ctl -Arguments @('wait', '30') | Out-Null
    $standing = Ctl -Arguments @('state')
    Ctl -Arguments @('fill', '-30', '79', '2', '-28', '80', '4', 'minecraft:air') | Out-Null
    Ctl -Arguments @('wait', '30') | Out-Null
    $fallen = Ctl -Arguments @('state')
    if (!$standing.player.on_ground -or $fallen.player.position[1] -ge $standing.player.position[1] - 1) {
        throw 'Collision cache did not release the player after removing the floor'
    }
    $result.collision_edit = @{ standing = $standing; fallen = $fallen }
    $result.passed = $true
    [IO.File]::WriteAllText($Output, ($result | ConvertTo-Json -Depth 40))
} finally {
    Ctl -Arguments @('setblock', '-26', '82', '8', $oldBlock.state) | Out-Null
    Ctl -Arguments @('fill', '-30', '79', '2', '-28', '80', '4', 'minecraft:stone') | Out-Null
    Ctl -Arguments @('command', 'gamemode creative') | Out-Null
    Ctl -Arguments @('tp', '-25.5', '82', '5.5') | Out-Null
    if ($present) { [IO.File]::WriteAllBytes($configPath, $original) }
    else { Remove-Item -LiteralPath $configPath -ErrorAction SilentlyContinue }
}
