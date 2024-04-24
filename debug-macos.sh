#!/bin/bash
FRESH_FLAG=""
if [ "$1" == "-f" ]; then
    FRESH_FLAG="--fresh"
fi

cd ./build || exit
export PATH="/opt/homebrew/opt/llvm/bin:$PATH" && \
export LDFLAGS="-L/opt/homebrew/opt/llvm/lib/c++ -Wl,-rpath,/opt/homebrew/opt/llvm/lib/c++" && \
export BUILD_TYPE=Debug && \
cmake -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
      -DBUILD_PRO=On \
      -DBUILD_SDLGPU=On \
      -DBUILD_NO_OPTIMIZATION=On \
      -DBUILD_ASAN_DEBUG=On \
      -DBUILD_LSAN_DEBUG=On \
      -DBUILD_UNDEFINED_DEBUG=On \
      -DCMAKE_C_COMPILER="$(which clang)" \
      -DCMAKE_CXX_COMPILER="$(which clang++)" \
       .. $FRESH_FLAG && \
       cmake --build . --config "$BUILD_TYPE" --parallel
