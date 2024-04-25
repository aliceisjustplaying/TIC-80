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
            DEBUG_FLAGS="-DBUILD_NO_OPTIMIZATION=On -DBUILD_ASAN_DEBUG=On -DBUILD_LSAN_DEBUG=On"
            ;;
    esac
done

cd ./build || exit

export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
export LDFLAGS="-L/opt/homebrew/opt/llvm/lib/c++ -Wl,-rpath,/opt/homebrew/opt/llvm/lib/c++ -L/opt/homebrew/opt/llvm/lib"
export CPPFLAGS="-I/opt/homebrew/opt/llvm/include"
export BUILD_TYPE=Debug

cmake -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
      -DBUILD_PRO=On \
      -DBUILD_SDLGPU=On \
      "$DEBUG_FLAGS" \
      -DCMAKE_C_COMPILER="$(which clang)" \
      -DCMAKE_CXX_COMPILER="$(which clang++)" \
       .. $FRESH_FLAG && \
       cmake --build . --config "$BUILD_TYPE" --parallel
