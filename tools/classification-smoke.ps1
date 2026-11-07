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
function Pick {
    param($Fixture)
    $x = $Fixture.x + 0.5
    Ctl -Arguments @('tp', "$x", '150', '-3.5') | Out-Null
    Ctl -Arguments @('look-at', "$x", '151.25', '0.5') | Out-Null
    (Ctl -Arguments @('state')).looking_at.block
}
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$configPath = Join-Path $InstanceHome 'config\nocubes.json'
$original = [IO.File]::ReadAllBytes($configPath)
if ((Ctl -Arguments @('state')).world -ne 'NoCubesValidation') { throw 'Use the isolated NoCubesValidation world' }
$fixtures = @(
    @{ x = 0; state = 'minecraft:glass'; smooth = $true },
    @{ x = 2; state = 'minecraft:oak_leaves'; smooth = $true },
    @{ x = 4; state = 'minecraft:oak_slab[type=bottom]'; smooth = $false },
    @{ x = 6; state = 'minecraft:oak_slab[type=double]'; smooth = $true },
    @{ x = 8; state = 'minecraft:short_grass'; smooth = $false },
    @{ x = 10; state = 'minecraft:torch'; smooth = $false },
    @{ x = 12; state = 'minecraft:oak_fence'; smooth = $false }
)
$old = @()
$result = [ordered]@{ version = '0.2.5'; fixtures = @() }
try {
    Ctl -Arguments @('command', 'gamemode spectator') | Out-Null
    foreach ($fixture in $fixtures) {
        foreach ($y in @(150, 151)) {
            $old += @{ x = $fixture.x; y = $y; state = (Ctl -Arguments @('block', "$($fixture.x)", "$y", '0')).state }
        }
        Ctl -Arguments @('setblock', "$($fixture.x)", '150', '0', 'minecraft:dirt') | Out-Null
        Ctl -Arguments @('setblock', "$($fixture.x)", '151', '0', $fixture.state) | Out-Null
    }
    [IO.File]::WriteAllText($configPath, '{}')
    Ctl -Arguments @('wait', '40') | Out-Null
    foreach ($fixture in $fixtures) { $fixture.enabled = Pick $fixture }
    [IO.File]::WriteAllText($configPath, '{"render":false}')
    Ctl -Arguments @('wait', '40') | Out-Null
    foreach ($fixture in $fixtures) {
        $fixture.disabled = Pick $fixture
        if (!$fixture.enabled -or !$fixture.disabled -or $fixture.enabled.pos[0] -ne $fixture.x -or
            $fixture.disabled.pos[0] -ne $fixture.x -or $fixture.enabled.pos[1] -ne 151 -or
            $fixture.disabled.pos[1] -ne 151) { throw "Fixture was not picked: $($fixture.state)" }
        $difference = [Math]::Abs($fixture.enabled.point[2] - $fixture.disabled.point[2])
        if (($fixture.smooth -and $difference -lt 0.01) -or (!$fixture.smooth -and $difference -gt 0.001)) {
            throw "Unexpected default geometry: $($fixture.state), difference $difference"
        }
        $result.fixtures += $fixture
    }
    $result.passed = $true
} finally {
    foreach ($block in ($old | Sort-Object y -Descending)) {
        Ctl -Arguments @('setblock', "$($block.x)", "$($block.y)", '0', $block.state) | Out-Null
    }
    [IO.File]::WriteAllBytes($configPath, $original)
    Ctl -Arguments @('wait', '30') | Out-Null
    [IO.File]::WriteAllText((Join-Path $root 'validation\classification-smoke.json'), ($result | ConvertTo-Json -Depth 20))
}
