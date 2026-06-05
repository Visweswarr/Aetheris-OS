param(
    [switch]$IncludeRoadmapTools
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$ConfigPath = Join-Path $Root "configs/polyglot-toolchain.toml"

if (!(Test-Path $ConfigPath)) {
    throw "missing toolchain config: $ConfigPath"
}

function Read-ToolVersions {
    $versions = @{}
    $inVersions = $false
    foreach ($line in Get-Content -LiteralPath $ConfigPath) {
        $trimmed = $line.Trim()
        if ($trimmed -eq "[versions]") {
            $inVersions = $true
            continue
        }
        if ($trimmed.StartsWith("[") -and $trimmed -ne "[versions]") {
            $inVersions = $false
        }
        if ($inVersions -and $trimmed -match '^([A-Za-z0-9_]+)\s*=\s*"([^"]+)"') {
            $versions[$Matches[1]] = $Matches[2]
        }
    }
    return $versions
}

$Versions = Read-ToolVersions
$ToolRoot = if ($env:POLYMERA_TOOL_ROOT) { $env:POLYMERA_TOOL_ROOT } else { Join-Path $Root ".polymera-tools" }
$CargoRoot = $ToolRoot
$NodePrefix = Join-Path $ToolRoot "node"
$PythonVenv = Join-Path $ToolRoot "python"
$Bin = Join-Path $ToolRoot "bin"
$NodeBin = Join-Path $NodePrefix "node_modules/.bin"
$PythonBin = Join-Path $PythonVenv "Scripts"

New-Item -ItemType Directory -Force -Path $ToolRoot, $Bin, $NodePrefix | Out-Null

function Install-CargoTool($Package, $Version) {
    Write-Host "installing $Package@$Version into $ToolRoot"
    cargo install $Package --version $Version --locked --root $CargoRoot
}

Install-CargoTool "wasm-tools" $Versions["wasm_tools"]
Install-CargoTool "wit-bindgen-cli" $Versions["wit_bindgen_cli"]
Install-CargoTool "cargo-component" $Versions["cargo_component"]

Write-Host "installing @bytecodealliance/jco@$($Versions["jco"]) into $NodePrefix"
npm install --prefix $NodePrefix "@bytecodealliance/jco@$($Versions["jco"])"

if (!(Test-Path $PythonVenv)) {
    python -m venv $PythonVenv
}
Write-Host "installing componentize-py==$($Versions["componentize_py"]) into $PythonVenv"
& (Join-Path $PythonBin "python.exe") -m pip install --upgrade pip
& (Join-Path $PythonBin "python.exe") -m pip install "componentize-py==$($Versions["componentize_py"])"

if ($IncludeRoadmapTools) {
    Write-Host "roadmap tools requested"
    Write-Host "Zig $($Versions["zig"]) is pinned; install manually from https://ziglang.org/download/ if this platform is not covered by your local package cache."
    if (Get-Command opam -ErrorAction SilentlyContinue) {
        opam install -y "fstar.$($Versions["fstar"])"
    } else {
        Write-Host "opam not found; skipping F* $($Versions["fstar"]) install"
    }
}

Write-Host ""
Write-Host "Add these directories to PATH for this shell when needed:"
Write-Host "  $Bin"
Write-Host "  $NodeBin"
Write-Host "  $PythonBin"
