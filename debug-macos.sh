#!/bin/bash
FRESH_FLAG=""
DEBUG_FLAGS=""

for arg in "$@"
do
    case $arg in
        -f)
            FRESH_FLAG="--fresh"
            ;;
        -d)
            DEBUG_FLAGS="-DBUILD_NO_OPTIMIZATION=On -DBUILD_ASAN_DEBUG=On"
            ;;
    esac
done

if [ "$FRESH_FLAG" == "--fresh" ]; then
    rm -rf .cache
    rm -rf CMakeCache.txt CMakeFiles/ cmake_install.cmake
    rm -rf build && git restore 'build/*'
fi

cd ./build || exit

export CPPFLAGS='-isystem/opt/local/include'
export LDFLAGS='-L/opt/local/lib'
export BUILD_TYPE=Debug

cmake -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
      -DBUILD_PRO=On \
      -DBUILD_SDLGPU=On \
      "$DEBUG_FLAGS" \
      -DCMAKE_C_COMPILER="$(which clang)" \
      -DCMAKE_CXX_COMPILER="$(which clang++)" \
       .. $FRESH_FLAG && \
       cmake --build . --config "$BUILD_TYPE" --parallel
