$ErrorActionPreference = "Stop"

$setup = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\setup.exe"
$installPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools"

Write-Host "Adding Windows SDK to Build Tools..."

$args = @(
    "modify",
    "--installPath", $installPath,
    "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22621",
    "--passive",
    "--norestart"
)

$p = Start-Process -FilePath $setup -ArgumentList $args -PassThru -Wait
Write-Host "Exit code: $($p.ExitCode)"

$kernel32 = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\Lib" -Recurse -Filter kernel32.lib -ErrorAction SilentlyContinue |
    Select-Object -First 1
if ($kernel32) {
    Write-Host "kernel32.lib: $($kernel32.FullName)"
} else {
    Write-Host "kernel32.lib not found yet"
}
