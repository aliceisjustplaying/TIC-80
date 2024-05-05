param(
    [switch]$f,
    [switch]$Fresh,
    [switch]$p,
    [switch]$Pro,
    [switch]$d,
    [switch]$Debug,
    [switch]$x,
    [switch]$Win32
)

$BUILD_TYPE = "MinSizeRel"
$SDLGPU_FLAG = "-DBUILD_SDLGPU=On"
$COMPILER_FLAGS = ""
$PRO_VERSION_FLAG = ""
$WIN32_FLAGS = ""

if ($f -or $Fresh) {
    Remove-Item -Path .cache, CMakeCache.txt, CMakeFiles, cmake_install.cmake -Force -Recurse -ErrorAction SilentlyContinue
    Remove-Item -Path build -Force -Recurse -ErrorAction SilentlyContinue
    git restore 'build/*'
}

if ($p -or $Pro) {
    $PRO_VERSION_FLAG = "-DBUILD_PRO=On"
}

if ($d -or $Debug) {
    $BUILD_TYPE = "Debug"
}

if ($x -or $Win32) {
    $WIN32_FLAGS = "-A Win32 -T v141_xp"
    Copy-Item -Path "build\janet\janetconf.h" -Destination "vendor\janet\src\conf\janetconf.h" -Force
    $SDLGPU_FLAG = ""
}

Write-Host
Write-Host "+--------------------------------+-------+"
Write-Host "|         Build Options          | Value |"
Write-Host "+--------------------------------+-------+"
Write-Host "| Fresh build (-f, --fresh)      | $(if ($f -or $Fresh) { "Yes" } else { "No " }) |"
Write-Host "| Pro version (-p, --pro)        | $(if ($p -or $Pro) { "Yes" } else { "No " }) |"
Write-Host "| Debug build (-d, --debug)      | $(if ($d -or $Debug) { "Yes" } else { "No " }) |"
Write-Host "| Win32 (-x, --win32)            | $(if ($x -or $Win32) { "Yes" } else { "No " }) |"
Write-Host "+--------------------------------+-------+"
Write-Host

if (-not (Test-Path -Path ./build)) {
    Write-Error "The 'build' directory does not exist. Please create it and try again."
    exit 1
}
Set-Location -Path ./build -ErrorAction Stop

cmake -DCMAKE_BUILD_TYPE="$BUILD_TYPE" `
      $PRO_VERSION_FLAG `
      $SDLGPU_FLAG `
      $COMPILER_FLAGS `
      $WIN32_FLAGS `
      -DCMAKE_EXPORT_COMPILE_COMMANDS=On `
      .. ;
if ($LASTEXITCODE -eq 0) {
    cmake --build . --config "$BUILD_TYPE" --parallel
}
