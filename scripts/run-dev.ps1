# Run voice dictation dev server from PowerShell
$ErrorActionPreference = "Stop"

# Strip Git usr/bin (GNU `link` breaks MSVC builds) from the shell we launch from
$cleanPath = ($env:Path -split ';' | Where-Object {
    $_ -and ($_ -notmatch '\\Git\\usr\\bin$') -and ($_ -notmatch '\\Git\\mingw64\\bin$')
}) -join ';'
$env:Path = "C:\Users\vince\.cargo\bin;$cleanPath"

$vcvarsCandidates = @(
    "${env:ProgramFiles}\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat",
    "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
    "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
)
$vcvars = $vcvarsCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

Set-Location $PSScriptRoot

if (-not $vcvars) {
    Write-Warning "MSVC Build Tools not found."
    Write-Warning "Install 'Desktop development with C++' (VS Build Tools or VS Community), then re-run."
    npm run tauri:dev
    exit $LASTEXITCODE
}

Write-Host "Using MSVC from: $vcvars"

function Import-VcVars64 {
    param([string]$BatchFile)
    cmd /c "`"$BatchFile`" >nul 2>&1 && set" | ForEach-Object {
        $eq = $_.IndexOf('=')
        if ($eq -gt 0) {
            $name = $_.Substring(0, $eq)
            $value = $_.Substring($eq + 1)
            [System.Environment]::SetEnvironmentVariable($name, $value, 'Process')
        }
    }
    $env:Path = "C:\Users\vince\.cargo\bin;" + (
        ($env:Path -split ';' | Where-Object {
            $_ -and ($_ -notmatch '\\Git\\usr\\bin$') -and ($_ -notmatch '\\Git\\mingw64\\bin$')
        }) -join ';'
    )
}

Import-VcVars64 -BatchFile $vcvars

Write-Host "Toolchain: $(rustc -vV | Select-String host)"
Write-Host "link.exe:  $((Get-Command link.exe -ErrorAction SilentlyContinue).Source)"
Write-Host "LIB set:   $($env:LIB -like '*Windows Kits*um*x64*')"

if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
    Write-Error "MSVC link.exe not on PATH after vcvars64. C++ workload may still be installing."
}

if ($env:LIB -notlike '*Windows Kits*um*x64*') {
    Write-Error "Windows SDK not found in LIB. Install the Windows 10/11 SDK via Visual Studio Installer."
}

npm run tauri:dev
exit $LASTEXITCODE
