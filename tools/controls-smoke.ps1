param(
    [Parameter(Mandatory = $true)][string]$Control,
    [string]$Socket = "$env:APPDATA\horizon-launcher\game-control.json",
    [string]$InstanceHome = "$env:APPDATA\horizon\instances\nocubes-port-validation",
    [string]$Output = "$PSScriptRoot\..\validation\controls-smoke.json",
    [string]$ScreenshotName = 'controls-unbound.png'
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
function Config { Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json }
function Settings {
    Ctl -Arguments @('options', 'mods') | Out-Null
    Ctl -Arguments @('click-button', 'Settings') | Out-Null
}
function Capture {
    param([string]$Key)
    Ctl -Arguments @('click-at', '20', '35') | Out-Null
    Ctl -Arguments @('hold', $Key, '1') | Out-Null
}
function Aim {
    param([string]$X)
    Ctl -Arguments @('look-at-block', $X, '151', '0') | Out-Null
    (Ctl -Arguments @('state')).looking_at.block
}
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$configPath = Join-Path $InstanceHome 'config\nocubes.json'
$original = [IO.File]::ReadAllBytes($configPath)
$state = Ctl -Arguments @('state')
if ($state.world -ne 'NoCubesValidation') { throw 'Open the isolated NoCubesValidation world first' }
$result = [ordered]@{ version = '0.2.6'; checks = @() }
$old = @{}
try {
    foreach ($x in @('0', '2', '4')) {
        $old[$x] = (Ctl -Arguments @('block', $x, '151', '0')).state
    }
    Ctl -Arguments @('command', 'gamemode spectator') | Out-Null
    Ctl -Arguments @('setblock', '0', '151', '0', 'minecraft:oak_planks') | Out-Null
    Ctl -Arguments @('setblock', '2', '151', '0', 'minecraft:oak_planks') | Out-Null
    Ctl -Arguments @('setblock', '4', '151', '0', 'minecraft:stone_bricks') | Out-Null
    Ctl -Arguments @('tp', '2.5', '150', '-3.5') | Out-Null
    Ctl -Arguments @('wait', '30') | Out-Null
    $result.first_smooth = Aim '0'
    $before = [IO.File]::ReadAllText($configPath)
    Ctl -Arguments @('hold', 'n', '1') | Out-Null
    if ([IO.File]::ReadAllText($configPath) -ne $before) { throw 'An old default key changed settings' }
    $result.checks += 'Old default key is inactive'
    Settings
    Ctl -Arguments @('screenshot', (Join-Path $root "validation\screenshots\$ScreenshotName")) | Out-Null
    Capture 'j'
    if ((Config).block_toggle_key -ne 'KeyJ') { throw 'Click-to-bind did not persist the chosen key' }
    Ctl -Arguments @('close') | Out-Null
    $result.first_smooth = Aim '0'
    $result.second_smooth = Aim '2'
    $result.other_type_smooth = Aim '4'
    Ctl -Arguments @('look-at-block', '0', '151', '0') | Out-Null
    Ctl -Arguments @('hold', 'j', '1') | Out-Null
    Ctl -Arguments @('wait', '10') | Out-Null
    if ((Config).blacklist -notcontains 'minecraft:oak_planks') { throw 'Block type was not disabled' }
    if ((Config).blacklist -contains 'minecraft:stone_bricks') { throw 'An unrelated type changed' }
    $result.first_cubic = Aim '0'
    $result.second_cubic = Aim '2'
    $result.other_type_unchanged = Aim '4'
    if (!$result.other_type_smooth -or !$result.other_type_unchanged -or
        [Math]::Abs($result.other_type_smooth.point[2] - $result.other_type_unchanged.point[2]) -gt 0.001) {
        throw 'An unrelated block type changed its picked surface'
    }
    foreach ($prefix in @('first', 'second')) {
        $smooth = $result["${prefix}_smooth"]
        $cubic = $result["${prefix}_cubic"]
        if (!$smooth -or !$cubic -or [Math]::Abs($smooth.point[2] - $cubic.point[2]) -lt 0.01) {
            throw "The $prefix block did not change its picked surface"
        }
    }
    $result.checks += 'One key disables both blocks of the same type, not other types'
    Ctl -Arguments @('look-at-block', '2', '151', '0') | Out-Null
    Ctl -Arguments @('hold', 'j', '1') | Out-Null
    if ((Config).blacklist -contains 'minecraft:oak_planks' -or
        (Config).whitelist -notcontains 'minecraft:oak_planks') { throw 'Block type did not toggle back on' }
    $result.checks += 'A second press restores smoothing for the type'
    Ctl -Arguments @('key', 'j', 'down') | Out-Null
    Ctl -Arguments @('key', 'j', 'repeat') | Out-Null
    Ctl -Arguments @('key', 'j', 'up') | Out-Null
    if ((Config).blacklist -notcontains 'minecraft:oak_planks') { throw 'A held/repeated key toggled more than once' }
    Ctl -Arguments @('hold', 'j', '1') | Out-Null
    $result.checks += 'Key repeats and releases do not toggle a second time'
    Settings
    Ctl -Arguments @('click-at', '20', '35') | Out-Null
    Ctl -Arguments @('key', 'shift', 'down') | Out-Null
    Ctl -Arguments @('key', 'j', 'tap') | Out-Null
    Ctl -Arguments @('key', 'shift', 'up') | Out-Null
    if ((Config).block_toggle_key -ne 'Shift+KeyJ') { throw 'Modifier chord capture failed' }
    Capture 'esc'
    if ((Config).block_toggle_key -ne 'Shift+KeyJ') { throw 'Cancelling capture replaced the existing chord' }
    $result.checks += 'Modifier capture and cancellation preserve the chosen binding'
    Capture 'backspace'
    if ((Config).block_toggle_key -ne '') { throw 'Clearing the binding did not persist an empty key' }
    Capture 'esc'
    if ((Config).block_toggle_key -ne '') { throw 'Cancelling capture changed the binding' }
    Ctl -Arguments @('close') | Out-Null
    $before = [IO.File]::ReadAllText($configPath)
    Ctl -Arguments @('hold', 'j', '1') | Out-Null
    if ([IO.File]::ReadAllText($configPath) -ne $before) { throw 'Cleared key still changes blocks' }
    $result.checks += 'Clearing/cancelling works and the cleared key is inactive'
    $messages = Ctl -Arguments @('messages')
    $result.messages = $messages
    if (($messages | ConvertTo-Json -Depth 10) -match 'NoCubes:.*toggles') { throw 'Old shortcut hints appeared in chat' }
    $result.checks += 'No shortcut hints are posted to chat'
    $result.passed = $true
} finally {
    Ctl -Arguments @('close') | Out-Null
    foreach ($x in $old.Keys) { Ctl -Arguments @('setblock', $x, '151', '0', $old[$x]) | Out-Null }
    [IO.File]::WriteAllBytes($configPath, $original)
    Ctl -Arguments @('wait', '30') | Out-Null
    [IO.File]::WriteAllText([IO.Path]::GetFullPath($Output), ($result | ConvertTo-Json -Depth 20))
}
