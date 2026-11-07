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
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$configPath = Join-Path $InstanceHome 'config\nocubes.json'
$original = [IO.File]::ReadAllBytes($configPath)
$distance = (Ctl -Arguments @('options-get', 'renderDistance')).options.renderDistance.value
$screenshots = Join-Path $root 'validation\screenshots\grass-0.2.3'
[IO.Directory]::CreateDirectory($screenshots) | Out-Null
$result = [ordered]@{ version = '0.2.3'; render_distance = 8; modes = @(); visual_review_required = $true }
try {
    $initial = Ctl -Arguments @('state')
    if ($initial.world -ne 'NoCubesValidation') { throw 'Open the isolated NoCubesValidation world first' }
    # StupidCubic intentionally emits internal faces; keep this visual regression
    # small enough to fit the GPU while testing its texture orientation too.
    Ctl -Arguments @('options-set', 'renderDistance', '8') | Out-Null
    Ctl -Arguments @('command', 'gamemode spectator') | Out-Null
    # Floating stepped grass fixture: flat tops, slopes, four cliff directions,
    # and exposed dirt bottoms, away from the natural terrain below.
    Ctl -Arguments @('fill', '-32', '110', '-8', '-24', '112', '0', 'minecraft:grass_block') | Out-Null
    Ctl -Arguments @('fill', '-23', '110', '-8', '-20', '113', '0', 'minecraft:grass_block') | Out-Null
    Ctl -Arguments @('fill', '-19', '110', '-8', '-16', '114', '0', 'minecraft:grass_block') | Out-Null
    Ctl -Arguments @('tp', '-28', '117', '-16') | Out-Null
    Ctl -Arguments @('look', '-20', '25') | Out-Null
    foreach ($mesher in @('SurfaceNets', 'SurfaceNets2x', 'MarchingCubes', 'MarchingCubes2x',
            'CullingCubic', 'StupidCubic', 'OldNoCubes', 'CullingChamfer', 'Wulferis')) {
        [IO.File]::WriteAllText($configPath, ('{"mesher":"' + $mesher + '"}'))
        Ctl -Arguments @('wait', '30') | Out-Null
        $status = Ctl -Arguments @('wait-ready', '60')
        Ctl -Arguments @('screenshot', (Join-Path $screenshots "$mesher.png")) | Out-Null
        $result.modes += @{ mesher = $mesher; pending_chunks = $status.pending_chunks; screenshot = "$mesher.png" }
        Write-Output "${mesher}: screenshot captured, $($status.pending_chunks) pending chunks"
    }
    [IO.File]::WriteAllText($configPath, '{"mesher":"SurfaceNets"}')
    Ctl -Arguments @('wait', '30') | Out-Null
    Ctl -Arguments @('wait-ready', '60') | Out-Null
    foreach ($view in @(
        @{ name = 'east'; position = @('-10', '115', '-4'); look = @('90', '10') },
        @{ name = 'south'; position = @('-24', '115', '8'); look = @('180', '10') },
        @{ name = 'west'; position = @('-38', '115', '-4'); look = @('-90', '10') },
        @{ name = 'bottom'; position = @('-24', '106', '-4'); look = @('0', '-80') },
        @{ name = 'hills'; position = @('-28.5', '87', '0.5'); look = @('-30', '40') }
    )) {
        Ctl -Arguments (@('tp') + $view.position) | Out-Null
        Ctl -Arguments (@('look') + $view.look) | Out-Null
        Ctl -Arguments @('wait', '3') | Out-Null
        Ctl -Arguments @('screenshot', (Join-Path $screenshots ($view.name + '.png'))) | Out-Null
    }
    $result.runtime_capture_passed = $true
    [IO.File]::WriteAllText((Join-Path $root 'validation\grass-texture-smoke.json'), ($result | ConvertTo-Json -Depth 10))
} finally {
    [IO.File]::WriteAllBytes($configPath, $original)
    Ctl -Arguments @('options-set', 'renderDistance', "$distance") | Out-Null
}
