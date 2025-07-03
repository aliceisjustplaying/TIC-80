# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TIC-80 is a free and open source fantasy computer for making, playing and sharing tiny games. It provides a complete development environment with built-in editors for code, sprites, maps, sound effects, and music.

## Build Commands

### macOS
```bash
# Recommended: Use the debug script
./debug-macos.sh        # Standard build
./debug-macos.sh -f     # Fresh build (clean first)
./debug-macos.sh -p     # Build PRO version
./debug-macos.sh -d     # Debug build
./debug-macos.sh -a     # Build with Address Sanitizer
./debug-macos.sh -s     # Static build
./debug-macos.sh -w     # Build with warnings

# Manual build
cd build
cmake -DBUILD_WITH_ALL=On ..
make -j4
```

### Key CMake Options
- `BUILD_WITH_ALL`: Build with all language support (recommended)
- `BUILD_PRO`: Build the PRO version with additional features
- `BUILD_SDLGPU`: Enable SDL GPU support
- `BUILD_STATIC`: Build static runtime
- `BUILD_TOOLS`: Build additional tools (bin2txt, prj2cart)

## Architecture

### Core Components

1. **src/api/**: Language bindings for Lua, JavaScript, Python, Ruby, Janet, Wren, Fennel, Moonscript, Squirrel
   - Each language has its own subdirectory with binding implementations
   - All languages share the same core API functions

2. **src/core/**: Fantasy computer core
   - `tic80.c`: Main TIC-80 runtime
   - `draw.c`: Rendering functions
   - `sound.c`: Audio synthesis
   - `io.c`: Input/output handling

3. **src/studio/**: Development environment
   - `studio.c`: Main studio orchestration
   - `editors/`: Individual editors (code.c, sprite.c, map.c, music.c, sfx.c)
   - `screens/`: UI screens (console.c, menu.c, surf.c, config.c)

4. **src/system/**: Platform-specific implementations
   - `sdl/`: Primary desktop implementation
   - `libretro/`: RetroArch core
   - `n3ds/`: Nintendo 3DS port
   - `baremetalpi/`: Raspberry Pi bare metal

### Key Concepts

- **Cartridge System**: Games are packaged as single .tic files containing code, graphics, maps, sounds, and music
- **Technical Constraints**: 240x136 display, 16 colors from 24-bit palette, 256 8x8 sprites, 240x136 map, 4 sound channels
- **Memory Map**: Fixed memory layout at 0x00000-0x17FFF including RAM, VRAM, and persistent storage
- **API Functions**: Core set of ~80 functions available across all languages (cls, spr, map, pix, clip, print, etc.)

### Development Workflow

1. **Adding Features**: Most features involve changes to both core (src/core/) and studio (src/studio/)
2. **Language Support**: New language bindings go in src/api/ and require CMake integration
3. **Platform Ports**: New platforms implement src/system/ interface
4. **Editor Improvements**: Each editor in src/studio/editors/ is relatively self-contained

### Running Tests

TIC-80 uses demo cartridges as integration tests:
```bash
# Build and run TIC-80
./build/bin/tic80

# In the TIC-80 console, load and run demos:
# load demos/tetris.lua
# run
```

Performance benchmarks are in demos/bunny/ for comparing language implementations.

## FFT Implementation

TIC-80 includes FFT support for audio visualization and livecoding:

### FFT Architecture
- **src/ext/fft.c**: Audio capture and FFT processing using KISS FFT
- **src/fftdata.h/c**: FFT data storage (1024 bins, smoothing, normalization)
- **Lua API**: `fft(freq)` and `ffts(freq)` functions for raw/smoothed data

### Key Details
- Uses miniaudio for audio capture (mic or loopback on Windows)
- 44100 Hz sample rate, 2048 sample buffer
- 1024 frequency bins (0-22050 Hz, ~21.5 Hz per bin)
- Auto-gain control with peak detection
- Smoothing factor: 0.6 for visual stability
- Processing in `tic_core_tick` before script execution

### Audio Flow
1. Audio capture via miniaudio callbacks
2. Stereo to mono conversion
3. KISS FFT real-to-complex transform
4. Magnitude calculation and normalization
5. Smoothing and peak tracking
6. Data available to scripts via API

## CQT Implementation Plan

### Overview
Add Constant-Q Transform (CQT) support alongside existing FFT for better musical frequency analysis with logarithmic frequency spacing and constant Q factor across octaves.

### Design Specifications

#### Parameters
- **Octaves**: 10 (20 Hz - 20 kHz full audio spectrum)
- **Bins per octave**: 12 (chromatic scale)
- **Total bins**: 120
- **Q factor**: ~17 (typical for 12 bins/octave)
- **Minimum frequency**: 20 Hz (below piano's lowest A)
- **Maximum frequency**: 20480 Hz (nearest note to 20 kHz)

#### API Design
```lua
-- Direct bin access (0-119)
value = cqt(bin)         -- Get raw CQT magnitude for bin
value = cqts(bin)        -- Get smoothed CQT magnitude for bin

-- Musical note access (octave 0-9, note 0-11)
value = cqto(octave, note)   -- Get raw CQT for specific note
value = cqtos(octave, note)  -- Get smoothed CQT for specific note

-- Note mapping: C=0, C#=1, D=2, D#=3, E=4, F=5, F#=6, G=7, G#=8, A=9, A#=10, B=11
```

### Implementation Architecture

#### File Structure
```
src/ext/
├── fft.c          (existing FFT implementation)
├── cqt.c          (new CQT implementation)
└── cqt_kernel.c   (CQT kernel generation from ESP32 reference)

src/
├── fftdata.h/c    (existing FFT data storage)
├── cqtdata.h/c    (new CQT data storage)

src/api/
└── luaapi.c       (add cqt(), cqts(), cqto(), cqtos() functions)
```

#### Data Structures (cqtdata.h)
```c
#define CQT_BINS 120
#define CQT_OCTAVES 10
#define CQT_BINS_PER_OCTAVE 12

typedef struct {
    float cqtData[CQT_BINS];              // Raw CQT magnitudes
    float cqtSmoothingData[CQT_BINS];    // Smoothed CQT data
    float cqtNormalizedData[CQT_BINS];   // Normalized (0-1) data
    bool cqtEnabled;                      // Enable flag (tied to fftEnabled initially)
    
    // Sparse kernel storage from ESP32 approach
    float* kernelReal[CQT_BINS];         // Real parts of kernels
    float* kernelImag[CQT_BINS];         // Imaginary parts
    int* kernelIndices[CQT_BINS];        // Non-zero indices
    int kernelLengths[CQT_BINS];         // Number of non-zero elements
} CqtData;
```

### Updated Design (Electronic Music Focus)

#### FFT Size Change
- **Use 4096-point FFT** instead of 2048
- Bin spacing: 44100 / 4096 = 10.76 Hz (better for sub-bass)
- Critical for electronic music's 20-60 Hz range
- Extra ~0.5ms overhead acceptable

### Processing Pipeline

1. **Initialization** (startup):
   - Pre-compute CQT kernels using Hamming window
   - Apply sparsity threshold (0.01) to reduce memory
   - Store only non-zero kernel values

2. **Runtime Processing** (per frame):
   - Share audio buffer from existing capture
   - Perform separate 4096-point FFT for CQT
   - Apply sparse kernels to FFT output
   - Calculate magnitudes and normalize
   - Apply smoothing (factor 0.7 for stability)

3. **Memory Optimization**:
   - Sparse kernels: ~30-40% of full matrix size
   - Estimated memory: ~200KB for kernels + 2KB for data
   - Acceptable for fantasy computer constraints

### Performance Considerations

#### Computational Cost
- One 4096-point FFT: ~1ms on modern CPU
- Sparse kernel application: ~0.5-1ms
- Total overhead: 1.5-2ms per frame (acceptable)

#### Optimization Strategies
1. Use SIMD where available (via compiler)
2. Process every other frame if needed
3. Cache kernel computations at startup
4. Use integer math for index calculations

### Integration Points

1. **Core Integration** (tic80.c):
   - Process CQT in `tic_core_tick` after FFT
   - Share `fftEnabled` flag initially
   - Add separate `cqtEnabled` flag later

2. **Build System** (CMakeLists.txt):
   - Add cqt.c and cqt_kernel.c to source list
   - No new dependencies (uses existing KISS FFT)

3. **Testing Strategy**:
   - Create demo showing CQT vs FFT comparison
   - Verify musical note accuracy with test tones
   - Benchmark on various platforms

### Rapid Implementation Plan (Test Early)

#### Phase 1: Minimal CQT (Test in 1-2 hours)
1. **Create cqtdata.h/c** with basic data arrays
2. **Copy/adapt ESP32 kernel generation** to cqt_kernel.c
3. **Create minimal cqt.c** that:
   - Generates kernels on init
   - Reuses existing audio buffer
   - Performs 4096-point FFT
   - Applies kernels to get CQT
4. **Add single test function** in luaapi.c: `cqt(bin)`
5. **Hook into tic_core_tick** after FFT processing
6. **Test with simple Lua visualization**

#### Phase 1 Status: COMPLETE ✓
- [x] cqtdata.h - Basic structure and constants
- [x] cqtdata.c - Global data instance  
- [x] cqt_kernel.h - Kernel generation header
- [x] cqt_kernel.c - Adapt ESP32 kernel generation
- [x] cqt.h - Main CQT header
- [x] cqt.c - Basic CQT processing (with placeholder FFT mapping)
- [x] luaapi.c - Add cqt() function
- [x] tic80.c - Hook into core tick
- [x] CMakeLists.txt - Add new files to build

**Test Results**: Basic API working, visualization shows 10 octaves with test data mapping

#### Phase 2: Real CQT Processing (Priority Tasks)
1. **Access raw audio buffer** from FFT capture system
2. **Implement 4096-point FFT** for CQT (separate from main FFT)
3. **Apply CQT kernels** to FFT output (currently stubbed)
4. **Fix kernel initialization** - kernels not being generated on startup
5. **Add remaining API functions**: `cqts()`, `cqto()`, `cqtos()`
6. **Create comparison demo** showing FFT vs CQT side-by-side

#### Phase 3: Optimization (If needed)
1. Profile and identify bottlenecks
2. Implement sparse kernel optimizations
3. Consider frame-skipping for lower-end hardware

### Test Script Example
```lua
-- Quick CQT test visualization
function TIC()
  cls(0)
  -- Draw CQT bins as bars
  for i=0,119 do
    local val = cqt(i) * 100  -- scale up for visibility
    rect(i*2, 136-val, 2, val, 12)
  end
  
  -- Draw octave markers
  for oct=0,9 do
    local x = oct * 12 * 2
    line(x, 0, x, 136, 8)
    print(oct, x+2, 2, 12, false, 1, true)
  end
  
  -- Show info
  print("CQT TEST - 10 octaves x 12 notes", 2, 120, 12, false, 1, true)
end
```

### Future Enhancements
1. Configurable octave range (6-10 octaves)
2. Variable bins per octave (24 for quarter-tones)
3. Separate enable flag for CQT
4. Phase information access
5. Inverse CQT for resynthesis

## Important Notes

- The project supports multiple scripting languages, all exposed through the same API
- Platform-specific code should be isolated in src/system/
- The studio uses immediate mode GUI principles
- Cartridge format is documented in wiki and src/studio/project.c
- PRO version enables additional features like extra memory banks
