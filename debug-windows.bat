@echo off

set BUILD_TYPE=MinSizeRel
set SDLGPU_FLAG=-DBUILD_SDLGPU=On
set COMPILER_FLAGS=
set PRO_VERSION_FLAG=
set WIN32_FLAGS=

set FRESH=false
set PRO=false
set DEBUG=false
set WIN32=false

:parse_args
if "%1"=="-f" set FRESH=true
if "%1"=="--fresh" set FRESH=true
if "%1"=="-p" set PRO=true
if "%1"=="--pro" set PRO=true
if "%1"=="-d" set DEBUG=true
if "%1"=="--debug" set DEBUG=true
if "%1"=="-x" set WIN32=true
if "%1"=="--win32" set WIN32=true
shift
if not "%1"=="" goto parse_args

if "%FRESH%"=="true" (
    rmdir /s /q .cache CMakeCache.txt CMakeFiles cmake_install.cmake 2>nul
    rmdir /s /q build 2>nul
    git restore "build/*"
)

if "%PRO%"=="true" set PRO_VERSION_FLAG=-DBUILD_PRO=On
if "%DEBUG%"=="true" set BUILD_TYPE=Debug
if "%WIN32%"=="true" (
    set WIN32_FLAGS=-A Win32 -T v141_xp
    copy "build\janet\janetconf.h" "vendor\janet\src\conf\janetconf.h" /Y
    set SDLGPU_FLAG=
)

call :set_value FRESH
call :set_value PRO
call :set_value DEBUG
call :set_value WIN32

echo.
echo +--------------------------------+-------+
echo ^|         Build Options          ^| Value ^|
echo +--------------------------------+-------+
echo ^| Fresh build (-f, --fresh)      ^| %FRESH_VALUE%    ^|
echo ^| Pro version (-p, --pro)        ^| %PRO_VALUE%    ^|
echo ^| Debug build (-d, --debug)      ^| %DEBUG_VALUE%    ^|
echo ^| Win32 (-x, --win32)            ^| %WIN32_VALUE%    ^|
echo +--------------------------------+-------+
echo.

if not exist build (
    echo The 'build' directory does not exist. Please create it and try again.
    exit /b 1
)
cd build

cmake -DCMAKE_BUILD_TYPE="%BUILD_TYPE%" ^
      %PRO_VERSION_FLAG% ^
      %SDLGPU_FLAG% ^
      %COMPILER_FLAGS% ^
      %WIN32_FLAGS% ^
      -DCMAKE_EXPORT_COMPILE_COMMANDS=On ^
      .. && ^
cmake --build . --config "%BUILD_TYPE%" --parallel

goto :eof

:set_value
if "%~1"=="true" (set %~1_VALUE=Yes) else (set %~1_VALUE=No)
goto :eof
