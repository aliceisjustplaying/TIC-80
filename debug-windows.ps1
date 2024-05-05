param(
    [switch]$f,
    [switch]$x
)

$FRESH_FLAG = ""

if ($f) {
    $FRESH_FLAG = "--fresh"
}

if ($FRESH_FLAG -eq "--fresh") {
    Remove-Item -Path .cache -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path CMakeCache.txt -Force -ErrorAction SilentlyContinue
    Remove-Item -Path CMakeFiles -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path cmake_install.cmake -Force -ErrorAction SilentlyContinue
    Remove-Item -Path build -Recurse -Force -ErrorAction SilentlyContinue
    git restore "build/*"
}

try {
if ($x) {
    Copy-Item -Path ".\build\janet\janetconf.h" -Destination ".\vendor\janet\src\conf\janetconf.h" -Force -ErrorAction Stop
    Set-Location .\build -ErrorAction Stop
    cmake -G "Visual Studio 16 2019" -A Win32 -T v141_xp -DCMAKE_BUILD_TYPE=MinSizeRel -DBUILD_WITH_MRUBY=Off -DBUILD_STUB=On .. $FRESH_FLAG
    if ($LASTEXITCODE -eq 0) {
        cmake --build . --config "MinSizeRel" --parallel
    }
}
else {
    if (-not $env:BUILD_TYPE) {
        $env:BUILD_TYPE = "Debug"
    }
    Set-Location .\build -ErrorAction Stop
    cmake -G "Visual Studio 16 2019" -DBUILD_SDLGPU=On -DCMAKE_BUILD_TYPE=$env:BUILD_TYPE .. $FRESH_FLAG
    if ($LASTEXITCODE -eq 0) {
        cmake --build . --config $env:BUILD_TYPE --parallel
    }
}
} catch {
    Set-Location ..
    Write-Error "An error occurred during the build process. Exiting the build directory."
    exit 1
}
