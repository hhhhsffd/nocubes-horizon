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
function Toggle-Player {
    Ctl -Arguments @('options', 'mods') | Out-Null
    Ctl -Arguments @('click-button', 'Settings') | Out-Null
    # The player-model option follows the unbound block-toggle row.
    Ctl -Arguments @('click-at', '20', '50') | Out-Null
    Ctl -Arguments @('close') | Out-Null
    Ctl -Arguments @('wait', '40') | Out-Null
}
function Shot {
    param([string]$Name)
    Ctl -Arguments @('screenshot', (Join-Path $screenshots "$Name.png")) | Out-Null
}
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$screenshots = Join-Path $root 'validation\screenshots\player-0.2.6'
[IO.Directory]::CreateDirectory($screenshots) | Out-Null
$configPath = Join-Path $InstanceHome 'config\nocubes.json'
$original = [IO.File]::ReadAllBytes($configPath)
$initial = Ctl -Arguments @('state')
if ($initial.world -ne 'NoCubesValidation') { throw 'Open the isolated NoCubesValidation world first' }
$result = [ordered]@{ version = '0.2.6'; visual_review_required = $true }
$camera = 0
try {
    $cfg = Get-Content $configPath -Raw | ConvertFrom-Json
    if ($cfg.player_model) { throw 'Start with the default player-model option off' }
    Ctl -Arguments @('command', 'gamemode creative') | Out-Null
    Ctl -Arguments @('tp', '-25.5', '82', '5.5') | Out-Null
    Ctl -Arguments @('look', '0', '0') | Out-Null
    Ctl -Arguments @('select', '9') | Out-Null
    Ctl -Arguments @('wait', '40') | Out-Null
    Ctl -Arguments @('wait-ready', '90') | Out-Null
    Ctl -Arguments @('key', 'f5', 'tap') | Out-Null; $camera = 1
    Ctl -Arguments @('key', 'f5', 'tap') | Out-Null; $camera = 2
    Shot 'normal-front'
    Ctl -Arguments @('command', 'gamemode survival') | Out-Null
    Ctl -Arguments @('wait', '5') | Out-Null
    Ctl -Arguments @('open-inventory') | Out-Null
    Shot 'normal-inventory'
    Ctl -Arguments @('close') | Out-Null
    Ctl -Arguments @('command', 'gamemode creative') | Out-Null
    $before = Ctl -Arguments @('status')
    Toggle-Player
    if (!(Get-Content $configPath -Raw | ConvertFrom-Json).player_model) { throw 'Settings toggle did not persist' }
    $enabled = Ctl -Arguments @('status')
    if ($enabled.meshed_sections -ne $before.meshed_sections) { throw 'Player-only toggle unexpectedly remeshed terrain' }
    Shot 'rounded-front'
    Ctl -Arguments @('command', 'gamemode survival') | Out-Null
    Ctl -Arguments @('wait', '5') | Out-Null
    Ctl -Arguments @('open-inventory') | Out-Null
    Shot 'rounded-inventory'
    Ctl -Arguments @('close') | Out-Null
    Ctl -Arguments @('command', 'gamemode creative') | Out-Null
    Ctl -Arguments @('move', 'forward', '0.5') | Out-Null
    Shot 'rounded-moving'
    Ctl -Arguments @('key', 'f5', 'tap') | Out-Null; $camera = 0
    Shot 'rounded-hand'
    Toggle-Player
    if ((Get-Content $configPath -Raw | ConvertFrom-Json).player_model) { throw 'Settings toggle did not turn the model off' }
    Shot 'normal-hand'
    $result.before = $before
    $result.enabled = $enabled
    $result.persisted_toggle_passed = $true
    $result.no_terrain_remesh_passed = $true
    $result.runtime_passed = $true
} finally {
    Ctl -Arguments @('close') | Out-Null
    while ($camera -ne 0) { Ctl -Arguments @('key', 'f5', 'tap') | Out-Null; $camera = ($camera + 1) % 3 }
    Ctl -Arguments @('command', 'gamemode', $initial.gamemode) | Out-Null
    Ctl -Arguments @('tp', "$($initial.player.position[0])", "$($initial.player.position[1])", "$($initial.player.position[2])") | Out-Null
    Ctl -Arguments @('look', "$($initial.player.yaw)", "$($initial.player.pitch)") | Out-Null
    Ctl -Arguments @('select', "$($initial.selected)") | Out-Null
    [IO.File]::WriteAllBytes($configPath, $original)
    Ctl -Arguments @('wait', '30') | Out-Null
    [IO.File]::WriteAllText((Join-Path $root 'validation\player-smoke.json'), ($result | ConvertTo-Json -Depth 15))
}
