[CmdletBinding()]
param(
    [Parameter()]
    [Alias("f")]
    [switch]$Fresh,  # Alias 'f' for fresh builds

    [Parameter()]
    [Alias("p")]
    [switch]$Pro,   # Alias 'p' for Pro version

    [Parameter()]
    [Alias("d")]
    [switch]$bDebug, # Alias 'd' for Debug build

    [Parameter()]
    [Alias("x")]
    [switch]$Win32,  # Alias 'x' for Win32 builds

    [Parameter()]
    [Alias("m")]
    [switch]$MRuby   # Alias 'm' for MRuby builds
)

$BUILD_TYPE = "MinSizeRel"
$SDLGPU_FLAG = "-DBUILD_SDLGPU=On"
$COMPILER_FLAGS = ""
$PRO_VERSION_FLAG = ""
$ARCH_FLAGS = "-A x64"
$MRUBY_FLAG = "-DBUILD_WITH_MRUBY=Off"

if ($Fresh) {
    Remove-Item -Path .cache, CMakeCache.txt, CMakeFiles, cmake_install.cmake -Force -Recurse -ErrorAction SilentlyContinue
    Remove-Item -Path build -Force -Recurse -ErrorAction Stop
    git restore 'build/*'
}

if ($Pro) {
    $PRO_VERSION_FLAG = "-DBUILD_PRO=On"
}

if ($bDebug) {
    $BUILD_TYPE = "Debug"
}

if ($Win32) {
    $ARCH_FLAGS = "-A Win32 -T v141_xp"
    Copy-Item -Path "build\janet\janetconf.h" -Destination "vendor\janet\src\conf\janetconf.h" -Force
    $SDLGPU_FLAG = ""
}

if ($MRuby) {
    $MRUBY_FLAG = "-DBUILD_WITH_MRUBY=On"
}

Write-Host
Write-Host "+--------------------------------+-------+"
Write-Host "|         Build Options          | Value |"
Write-Host "+--------------------------------+-------+"
Write-Host "| Fresh build (-f, --fresh)      | $(if ($Fresh) { "Yes" } else { "No " })   |"
Write-Host "| Pro version (-p, --pro)        | $(if ($Pro) { "Yes" } else { "No " })   |"
Write-Host "| Debug build (-d, --debug)      | $(if ($bDebug) { "Yes" } else { "No " })   |"
Write-Host "| Win32 (-x, --win32)            | $(if ($Win32) { "Yes" } else { "No " })   |"
Write-Host "| Build with MRuby (-m, --mruby) | $(if ($MRuby) { "Yes" } else { "No " })   |"
Write-Host "+--------------------------------+-------+"
Write-Host

if (-not (Test-Path -Path ./build)) {
    Write-Error "The 'build' directory does not exist. Please create it and try again."
    exit 1
}

Set-Location -Path ./build -ErrorAction Stop

# Construct the command
$cmakeCommand = "cmake -G `"Visual Studio 16 2019`" " +
                "$ARCH_FLAGS " +
                "$MRUBY_FLAG " +
                "-DCMAKE_BUILD_TYPE=$BUILD_TYPE " +
                "$PRO_VERSION_FLAG " +
                "$SDLGPU_FLAG " +
                "-DCMAKE_EXPORT_COMPILE_COMMANDS=On " +
                ".."

# Debug output
Write-Host "Executing CMake Command: $cmakeCommand"

# Execute the command
Invoke-Expression $cmakeCommand

if ($LASTEXITCODE -eq 0) {
    cmake --build . --config "$BUILD_TYPE" --parallel
}

Set-Location -Path .. -ErrorAction Stop
