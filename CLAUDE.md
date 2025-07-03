# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TIC-80 is a free and open source fantasy computer for making, playing and sharing tiny games. It provides a complete development environment with built-in editors for code, sprites, maps, sound effects, and music.

## Build Commands

**IMPORTANT: Never run `./debug-macos.sh` or any build commands unless explicitly instructed by the user. The user will handle building and testing.**

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

#### Phase 2: Real CQT Processing (COMPLETE)
1. ✓ **Access raw audio buffer** from FFT capture system
2. ✓ **Implement 4096-point FFT** for CQT (separate from main FFT)
3. ✓ **Apply CQT kernels** to FFT output
4. ✓ **Fix kernel initialization** - kernels are properly generated on startup
5. ✓ **Debug and fix frequency mapping** - CQT now correctly detects frequencies
6. **Add remaining API functions**: `cqts()`, `cqto()`, `cqtos()` (TODO)
7. **Create comparison demo** showing FFT vs CQT side-by-side (TODO)

**TEMPORARY CHANGE**: FFT_SIZE has been changed from 1024 to 2048 in fftdata.h to support CQT's 4096-point FFT requirement. This temporarily breaks FFT resolution (now 2048 bins instead of 1024) but allows CQT to function properly. This should be reverted once a separate audio buffer is implemented for CQT.

**CURRENT ISSUE**: CQT frequency mapping is incorrect. Test shows:
- Playing 110 Hz (A2) appears at bin 0-4 instead of expected bin 30
- Error of ~26-30 bins (2-3 octaves too low)
- CQT is detecting signals but at wrong frequency bins

**Debugging Status**:
1. Changed smoothing from 0.7 to 0.3 - improved peak sharpness ✓
2. Fixed window length calculation to use Q factor ✓ 
3. Fixed kernel generation to use equal temperament spacing ✓
4. Added gain boost (4x) for better visibility ✓
5. Created test scripts: test_cqt_tone.lua, test_cqt_debug.lua ✓
6. **FIXED kernel phase calculation** - Critical fix! ✓
   - Problem: Was using `(i - windowLength/2) / sampleRate` for phase
   - Solution: Use `(idx - fftSize/2) * (centerFreq / sampleRate)` like ESP32
   - The modulation must be based on position in full FFT buffer, not window

**Key Fix Applied**:
```c
// OLD (incorrect):
float t = (i - windowLength/2.0f) / sampleRate;
float phase = 2.0f * M_PI * centerFreq * t;

// NEW (correct - matches ESP32):
float phase = 2.0f * M_PI * (centerFreq / sampleRate) * (idx - fftSize/2);
```

This should fix the frequency mapping issue. Ready for testing!

**Update after testing**: 
- The phase fix helped but frequencies are still mapping to wrong bins
- 110 Hz appears at bin 3 instead of 30 (27-bin error)
- Created test_cqt_stable.lua for controlled testing
- Created test_cqt_a4.lua for constant 440Hz tone
- Added debug output to show kernel FFT bin ranges
- Issue appears to be in kernel generation or application

**Current hypothesis**:
The kernels might be centered at the wrong FFT bins. Need to verify:
1. The FFT bin calculation for each center frequency
2. The kernel's actual FFT bin coverage
3. Whether the complex multiplication is being done correctly

**Issues found**:
1. Audio generation in test cart was wrong - fixed in test_cqt_a4.lua
2. Added debug output to show:
   - FFT bin magnitudes around 440 Hz (should peak at bin 41)
   - Which CQT bins have high values
   - This will help identify if the issue is in kernel generation or application

**To test**: 
Build and run with test_cqt_a4.lua. The console should show:
- Every second: FFT bins around 440 Hz and active CQT bins
- This will reveal if 440 Hz is detected at FFT bin 41 but mapped to wrong CQT bin

**Debug results**:
- FFT correctly shows 440 Hz peak at bin 41 (magnitude 730) ✓
- CQT bin 54 has high value (27595) which is correct for 440 Hz ✓
- BUT: Almost ALL CQT bins (0-105) have significant energy
- This means kernels are not frequency-selective enough

**Fix applied**:
- Changed normalization to match ESP32: divide by windowLength before FFT
- Added scaling factor after FFT: multiply by centerFreq/minFreq
- This should make kernels more frequency-selective

**Test results after fix**:
- 440 Hz correctly peaks at CQT bin 54 ✓
- BUT: ALL bins (0-119) have significant values
- High frequency bins (90-119) have values in thousands!
- This means kernels are not properly bandpass filtered

**New hypothesis**:
The issue is that our kernels are "seeing" all frequencies. Possible causes:
1. The modulation in time domain might be creating aliases
2. The window might be too wide or too narrow
3. The FFT of the kernel might need different normalization

**Next debugging steps**:
- Removed scaling factor (made it worse)
- Added debug to show kernel 54's FFT bin range
- Need to verify kernels are properly bandpass filtered around their center frequencies

**SOLUTION FOUND**: 
- Removed the scaling factor - this was the key fix!
- Kernel 54 now correctly uses only 9 FFT bins (38-46) centered on bin 41
- 440 Hz correctly peaks at CQT bin 54 with magnitude 681
- Only ~50 bins have values > 0.1 (vs all 120 before)
- CQT is now properly frequency-selective!

**Working implementation**:
- Kernels use Q-factor based window length
- Normalize by windowLength before FFT
- NO scaling factor after FFT
- Proper phase calculation: `2π * (f/fs) * (idx - N/2)`

**Final Status**:
- CQT correctly detects frequencies (440 Hz → bin 54)
- Good frequency selectivity (minimal spreading)
- Kernels properly bandpass filtered
- 20 Hz start preserved for electronic music sub-bass
- Created test_cqt_spectrum_v2.lua with correct note display

**NEW ISSUE - Excessive Frequency Spreading**:
Testing with pure sine waves shows excessive frequency spreading:
- 20 Hz sine wave: Energy spreads across ~6-7 bins
- 50 Hz sine wave: Energy spreads across ~8-9 bins
- 100 Hz sine wave: Energy spreads across ~4-5 bins

## Root Cause Analysis:
The issue is **window truncation** in our current implementation:
- At 20 Hz: Window should be 37,485 samples but is clamped to 4,096 (line 90 in cqt_kernel.c)
- At 50 Hz: Window should be 14,994 samples but is clamped to 4,096
- At 100 Hz: Window should be 7,497 samples but is clamped to 4,096

This truncation reduces the effective Q factor:
- At 20 Hz: Effective Q ≈ 1.86 instead of 17
- At 50 Hz: Effective Q ≈ 4.64 instead of 17
- At 100 Hz: Effective Q ≈ 9.29 instead of 17

## Comparison: Our Approach vs ESP32 Approach

### Our Current Approach (Constant Q):
```c
windowLength = Q * sampleRate / centerFreq;  // Q ≈ 17
if (windowLength > fftSize) windowLength = fftSize;  // TRUNCATION!
```
- **Pros**: Excellent frequency resolution at high frequencies
- **Cons**: Severe truncation at low frequencies causing spreading

### ESP32 Approach (Variable Window):
```c
windowLength = fftSize / (centerFreq / minFreq);  // scales with 1/f
```
- **Pros**: All windows fit within FFT size, no truncation
- **Cons**: Very low Q (≈1.86) giving ~9.56 semitone bandwidth

### Analysis of ESP32 Approach:
- Constant effective Q ≈ 1.86 for all frequencies
- Bandwidth: ~956 cents (9.56 semitones) - can't distinguish adjacent notes
- At 20 Hz: 4096 samples (92.9ms)
- At 20 kHz: 4 samples (0.1ms) - too short for analysis

## Alternative Approaches Considered:

### 1. Larger FFT Size (16K or 32K):
- **16K FFT**: Handles windows down to ~45 Hz properly
- **Computational cost**: ~5-10ms (still acceptable for 60 FPS)
- **Memory**: ~2.4MB for 16K, ~4.9MB for 32K
- **Pros**: Full constant-Q accuracy
- **Cons**: Higher resource usage

### 2. Multi-Resolution CQT:
- Use 16K FFT for lowest 2 octaves (20-80 Hz)
- Use 4K FFT for everything else
- **Pros**: Best accuracy at all frequencies
- **Cons**: Complex implementation, multiple FFTs per frame

### 3. Adaptive Q Factor:
- Reduce Q for low frequencies to fit within FFT size
- **Pros**: Single FFT, reasonable accuracy
- **Cons**: Variable frequency resolution

## SELECTED SOLUTION: Hybrid Approach

Combine the best of both methods:
- **Below 100 Hz**: Use ESP32-style window scaling
- **Above 100 Hz**: Use constant-Q approach
- **Transition**: Smooth crossfade between methods

### Implementation Plan:

#### Step 1: Modify Window Length Calculation
In `cqt_kernel.c`, function `generateSingleKernel()` around line 86:
```c
// Current code to replace:
float Q = CQT_CalculateQ(CQT_BINS_PER_OCTAVE);
int windowLength = (int)(Q * sampleRate / centerFreq);

// New hybrid approach:
float Q = CQT_CalculateQ(CQT_BINS_PER_OCTAVE);
int windowLength;

if (centerFreq < 100.0f) {
    // ESP32-style for low frequencies
    float factor = centerFreq / minFreq;
    windowLength = (int)(fftSize / factor);
} else {
    // Constant-Q for higher frequencies
    windowLength = (int)(Q * sampleRate / centerFreq);
    
    // Ensure it fits in FFT size with some margin
    if (windowLength > fftSize * 0.9) {
        windowLength = (int)(fftSize * 0.9);
    }
}
```

#### Step 2: Adjust Normalization
The normalization may need adjustment based on actual window length used.

#### Step 3: Test and Validate
- Test with pure tones at 20, 50, 100, 200, 440, 1000 Hz
- Verify smooth transition at 100 Hz boundary
- Check musical accuracy across spectrum

### Expected Results:
- **20 Hz**: ~1-2 bin spread (using ESP32 method)
- **100 Hz**: ~1-2 bin spread (transition point)
- **440 Hz**: <1 bin spread (constant-Q)
- **Electronic music**: Good sub-bass and treble accuracy

### Future Improvements:
1. Smooth transition zone (80-120 Hz) instead of hard cutoff
2. Configurable transition frequency
3. Optional multi-resolution mode for maximum accuracy

## FFT Performance Measurements (CRITICAL UPDATE)

### M1 Pro Benchmark Results:
Actual measurements completely contradict initial estimates:
```
CQT FFT Benchmark on this CPU:
================================
 4096-point FFT: 0.014 ms
 6144-point FFT: 0.021 ms (current implementation)
 8192-point FFT: 0.025 ms
12288-point FFT: 0.047 ms
16384-point FFT: 0.066 ms (!!)
================================
```

### Key Findings:
- **75-150x faster than conservative estimates**
- 16K FFT uses only 0.066ms (0.4% of 16.67ms frame budget at 60fps)
- Even 32K FFT would likely be ~0.15ms (still under 1% frame budget)
- Apple Silicon (or auto-vectorization) provides exceptional FFT performance

### Performance Comparison Results:
- **M1 Pro**: 16K FFT = 0.066ms
- **Intel i5-1130G7 (ThinkPad X1 Nano)**:
  - Performance mode: 16K FFT = 0.112ms (0.67% of frame budget)
  - Power save mode: 16K FFT = 0.335ms (2% of frame budget)
- Even in power save mode, 16K FFT is completely viable!

### Implications:
**16K FFT is now implemented** and provides:
- 20Hz: Q≈7.4 (truncated from ideal 17, but much better than previous 1.86)
- 30Hz: Q≈11.2 (near ideal)
- 45Hz+: Full Q≈17 (ideal resolution - no truncation!)
- Dramatically improved low-frequency resolution for electronic music

The profiling code has been added to measure actual performance on each platform.

## Current CQT Implementation Status (December 2024)

### What's Working:
- **16K FFT implemented** - provides excellent low-frequency resolution
- **cqt(bin)** function working correctly - returns raw CQT magnitude for bin 0-119
- **Frequency detection accurate** - 440Hz correctly maps to bin 54, etc.
- **Kernels properly generated** - sparse storage working, good frequency selectivity
- **Performance excellent** - Runtime on M1 Pro: 0.368ms total (0.255ms FFT + 0.113ms kernels)

### Runtime Performance Expectations:
- **M1 Pro MacBook**: 0.368ms total CQT processing (~2.2% of frame budget)
- **ThinkPad i5-1130G7 (extrapolated)**:
  - Performance mode: ~0.426ms (2.6% of frame budget)
  - Power save mode: ~1.273ms (7.6% of frame budget)
- Runtime is ~3.8x slower than synthetic benchmarks due to real audio data, cache effects

### Critical Implementation Details:
1. **FFT_SIZE temporarily changed** in fftdata.h from 1024 to 8192 to support CQT
   - This breaks FFT resolution but enables CQT
   - Must be reverted when separate audio buffer is implemented
   
2. **Key fixes that were essential**:
   - Kernel phase calculation must use full FFT position: `2π * (f/fs) * (idx - N/2)`
   - NO scaling factor after kernel FFT - this was critical!
   - Normalization by windowLength before FFT
   
3. **Test scripts created**:
   - `test_cqt_spectrum_v2.lua` - visual spectrum analyzer
   - `test_cqt_a4.lua` - 440Hz tone generator
   - `test_cqt_stable.lua` - controlled testing

### Resolution Characteristics with 16K FFT:
- **20Hz**: Q≈7.4 (window truncated to 16384 samples, but much better than 6K's Q≈1.86)
- **30Hz**: Q≈11.2 (good resolution)
- **45Hz+**: Full Q≈17 (perfect - no truncation)

## Phase 3: Next Steps
1. ~~Implement 16K FFT based on benchmark results~~ (COMPLETE)
2. Add remaining API functions: `cqts()`, `cqto()`, `cqtos()`
3. Create separate audio buffer for CQT (restore FFT_SIZE to 1024)
4. Create FFT vs CQT comparison demo
5. Test and verify improved frequency resolution at 20Hz, 50Hz, 100Hz

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
