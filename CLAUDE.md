# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TIC-80 is a free and open source fantasy computer for making, playing and sharing tiny games. It provides a complete development environment with built-in editors for code, sprites, maps, sound effects, and music.

## Build Commands

**IMPORTANT: Never run `./debug-macos.sh` or any build commands unless explicitly instructed by the user. The user will handle building and testing.**

**IMPORTANT:** When given permission to build and test, use **THIS EXACT COMMAND**: ./debug-macos.sh -p -h -s

You will find the build output in `build/bin/tic80`.

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
- **FFT fully restored** - Returns 1024 bins with exact original behavior preserved
- **CQT with 8K FFT** - Optimized for responsive visualization (~5.4 fps)
- **Variable-Q Implementation** ✨ NEW! - Optimized Q values for 8K FFT constraint
- **Shared audio buffer** - Automatically sized to max(FFT needs, CQT needs)
- **cqt(bin)** function working - Returns raw CQT magnitude for bin 0-119
- **Frequency detection accurate** - 440Hz correctly maps to bin 54, etc.
- **Kernels properly generated** - Sparse storage, good frequency selectivity
- **Smoothing calculated** - But not exposed via API yet

### API Status:
| Function | Status | Description |
|----------|---------|-------------|
| `fft(bin)` | ✅ Complete | Raw FFT data (0-1023) |
| `ffts(bin)` | ✅ Complete | Smoothed FFT data |
| `cqt(bin)` | ✅ Complete | Raw CQT data (0-119) |
| `cqts(bin)` | ❌ TODO | Smoothed CQT data |
| `cqto(octave, note)` | ❌ TODO | Raw CQT by musical note |
| `cqtos(octave, note)` | ❌ TODO | Smoothed CQT by musical note |

### Variable-Q Implementation (8K FFT Optimized):
The Variable-Q implementation provides frequency-dependent Q factors optimized to fit within 8K FFT window size:

| Frequency Range | Design Q | Effective Q | Resolution | Notes |
|----------------|----------|-------------|------------|-------|
| 20-25 Hz | 7.4 | 3.7 | ~5.4 Hz | Limited by 8K FFT |
| 25-30 Hz | 9.2 | 4.6 | ~5.4 Hz | Better than fixed Q |
| 30-40 Hz | 11.5 | 5.6-7.4 | ~5.1 Hz | Good for bass |
| 40-50 Hz | 14.5 | 7.4-9.3 | ~4.3 Hz | Near ideal |
| 50-65 Hz | 16.0 | 9.3-12.1 | ~4.1 Hz | Almost full Q |
| 65-80 Hz | 17.0 | 12.1-17.0 | ~4.7 Hz | Full standard Q |
| 80+ Hz | 17.0 | 17.0 | Standard CQT | No truncation |

### Performance with 8K FFT:
- **Update rate**: ~5.4 fps (good for responsive visualization)
- **M1 Pro**: ~0.2ms total (1.2% of frame budget)
- **Bass resolution**: Much improved over fixed Q=17
- **All windows fit** within 8K samples above 80 Hz

### Key Benefits of 8K-Optimized Variable-Q:
- **Smooth Q transition**: Gradually increases from 7.4 to 17 across frequency range
- **No harsh cutoffs**: All windows designed to fit within 8K constraint
- **Better than fixed Q**: ~5 Hz resolution at 20 Hz (vs ~11 Hz with fixed Q=17)
- **Responsive updates**: Maintains 5.4 fps for livecoding applications
- **Electronic music optimized**: Good sub-bass resolution where it matters most

### Implementation Architecture:
```
Audio Capture (44.1kHz)
    ↓
Shared Buffer (8192 samples currently)
    ├─→ FFT: Uses first 2048 samples → 1024 bins
    └─→ CQT: Uses first 8192 samples → 120 bins
```

### Key Implementation Details:
1. **Buffer management**:
   - `AUDIO_BUFFER_SIZE` in fft.c automatically adjusts
   - FFT always reads samples 0-2047
   - CQT reads samples 0-(CQT_FFT_SIZE-1)

2. **Smoothing implementation**:
   - FFT: 0.6 factor (60% old, 40% new)
   - CQT: 0.3 factor (30% old, 70% new) - calculated but not exposed
   - Both use peak tracking with 0.99 decay

3. **Test scripts**:
   - `demo_fft_cqt_hybrid.lua` - Combined FFT/CQT visualization
   - `test_cqt_spectrum_v2.lua` - CQT spectrum analyzer
   - `test_fft_restored.lua` - FFT verification
   - `test_cqt_variable_q.lua` - Variable-Q demonstration (NEW!)

## Next Steps
1. ✅ ~~Implement configurable FFT for CQT~~ (COMPLETE - using 8K default)
2. ✅ ~~Create separate audio buffer for CQT~~ (COMPLETE - shared buffer)
3. ✅ ~~Restore FFT_SIZE to 1024~~ (COMPLETE)
4. ✅ ~~Implement Variable-Q for better bass resolution~~ (COMPLETE - 8K optimized)
5. ❌ Add remaining API functions: `cqts()`, `cqto()`, `cqtos()`
6. ❌ Add CQT to other language bindings (currently Lua only)
7. ❌ Create comprehensive FFT vs CQT comparison demo
8. ❌ Add configuration options for CQT parameters
9. ❌ Implement CQT enhancements (spectral whitening, HPS, etc.)

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

## FFT vs CQT: Understanding the Tradeoffs

### Fundamental Difference
- **FFT**: Linear frequency spacing (each bin = 21.5 Hz)
- **CQT**: Logarithmic frequency spacing (constant Q = ~17)

### Time-Frequency Resolution Tradeoff (Uncertainty Principle)
This is fundamental physics - to distinguish frequencies, you must observe for sufficient time:
- To tell 20 Hz from 21 Hz: Need 1 second of observation
- To tell 1000 Hz from 1001 Hz: Still need 1 second
- Shorter window = better time resolution, worse frequency resolution
- Longer window = better frequency resolution, worse time resolution

### FFT Characteristics
- **Window**: 2048 samples (46ms)
- **Update rate**: ~21 fps
- **Frequency resolution**: 21.5 Hz per bin (constant)
- **Best for**: Beat detection, rhythm visualization, transients
- **API**: `fft(bin)` raw, `ffts(bin)` smoothed (0.6 factor)

### CQT Characteristics  
- **Window**: Variable per frequency (up to 16384 samples)
- **Update rate**: ~5.4 fps (8K FFT) or ~2.7 fps (16K FFT)
- **Frequency resolution**: Constant Q≈17 (logarithmic spacing)
- **Best for**: Note detection, chord analysis, harmonic content
- **API**: `cqt(bin)` raw, `cqts(bin)` smoothed (0.3 factor) - smoothed not yet implemented

### Resolution Comparison

| Method | 40 Hz Resolution | 440 Hz Resolution | 4400 Hz Resolution |
|--------|------------------|-------------------|-------------------|
| FFT    | ±10.75 Hz (25%)  | ±10.75 Hz (2.4%)  | ±10.75 Hz (0.24%) |
| CQT    | ±1.4 Hz (3.5%)   | ±15 Hz (3.5%)     | ±150 Hz (3.5%)    |

### Configurable CQT FFT Sizes

| FFT Size | Update Rate | Low Freq Quality | Use Case |
|----------|-------------|------------------|----------|
| 4K | 10.8 fps | Poor (Q≈1.9 @ 20Hz) | Rhythm only, not musical |
| 8K | 5.4 fps | Decent (Q≈3.7 @ 20Hz) | Good balance for livecoding |
| 16K | 2.7 fps | Good (Q≈7.4 @ 20Hz) | Best accuracy, slow update |

#### 4K FFT Deep Dive (Not Recommended)
With only 4096 samples, the maximum achievable Q at each frequency is severely limited:

| Frequency | Max Q (4K) | Bandwidth | Musical Impact |
|-----------|------------|-----------|----------------|
| 20 Hz | 1.86 | 10.8 Hz | Can't distinguish notes in same octave |
| 40 Hz | 3.72 | 10.8 Hz | E1 and F1 merge together |
| 80 Hz | 7.43 | 10.8 Hz | ~1.5 semitone resolution |
| 160 Hz | 14.87 | 10.8 Hz | Approaching usable |
| 200+ Hz | 17+ | Standard | Full CQT resolution |

**Verdict**: 4K FFT turns CQT into a "bass energy detector" rather than note detector. The 10+ fps is tempting but the musical accuracy is too poor for practical use.

Current implementation uses 8K by default (configurable via `CQT_FFT_SIZE` in cqtdata.h), providing the best balance between update rate (5.4 fps) and frequency resolution.

### Smoothing Factors
- **FFT**: 0.6 (60% old, 40% new) - more stable
- **CQT**: 0.3 (30% old, 70% new) - more responsive
- Peak tracking uses 0.99 factor for slow decay

### Shared Audio Buffer Architecture
Both FFT and CQT share the same audio capture buffer:
- Buffer size: Maximum of (2048, CQT_FFT_SIZE) samples
- FFT reads samples 0-2047 (always the same)
- CQT reads samples 0-(CQT_FFT_SIZE-1)
- This preserves exact FFT behavior while allowing CQT flexibility

### Frame Rate vs BPM Limitations

#### CQT Update Rate Analysis (5.4 fps = 185ms per frame)

| BPM | Beat Duration | Beats per Frame | 16th Notes per Frame | Suitability |
|-----|---------------|-----------------|---------------------|-------------|
| 120 | 500ms | 0.37 beats | 1.5 sixteenths | ✅ Good |
| 128 | 469ms | 0.39 beats | 1.6 sixteenths | ✅ Good |
| 140 | 429ms | 0.43 beats | 1.7 sixteenths | ✅ Good |
| 150 | 400ms | 0.46 beats | 1.9 sixteenths | ✅ OK |
| 160 | 375ms | 0.49 beats | 2.0 sixteenths | ⚠️ Borderline |
| **174** | **345ms** | **0.54 beats** | **2.1 sixteenths** | **❌ Critical point** |
| 180 | 333ms | 0.56 beats | 2.2 sixteenths | ❌ Too fast |
| 200 | 300ms | 0.62 beats | 2.5 sixteenths | ❌ Unusable |

**At 174 BPM, CQT misses every other beat!**

#### Genre Suitability

- **House/Techno (120-130 BPM)**: ✅ Excellent - 2.5+ updates per beat
- **Dubstep (140 BPM)**: ✅ Good - 2.3 updates per beat
- **Trance (130-150 BPM)**: ✅ Mostly fine - 2+ updates per beat
- **Drum & Bass (160-180 BPM)**: ⚠️ Problematic - Fast breaks blur
- **Hardcore/Gabber (180-200+ BPM)**: ❌ Unusable - Complete desync

#### What Actually Breaks
- **Kick synchronization** fails above 170 BPM
- **Hi-hat patterns** (32nd notes) blur into continuous energy
- **Amen breaks** become unrecognizable smears
- **Bass wobbles** look stepped instead of smooth

### Practical Usage for Electronic Music Visualization

**Use FFT for:**
- Kick drum detection (bins 2-4, ~40-80 Hz)
- Beat synchronization
- Energy meters by frequency band
- Reactive elements needing >10 fps update
- **Any rhythm visualization above 150 BPM**

**Use CQT for:**
- Bass note identification
- Chord/key detection  
- Color mapping from musical content
- Melodic visualization
- **Harmonic content (not rhythm)**

**Hybrid Approach (Recommended):**
```lua
-- Rhythm from FFT (21 fps)
local kick = fft(2) + fft(3) + fft(4)

-- Musical content from CQT (5.4 fps)
local bassNote = 0
for i=24,35 do  -- 2nd octave
  if cqt(i) > cqt(bassNote) then bassNote = i end
end

-- Combine for visuals
local pulse = kick * 2  -- Size from rhythm
local color = (bassNote % 12) + 1  -- Color from note

-- For fast music, predict beats
local bpm = 175  -- D&B tempo
local beatPhase = (time() * bpm / 60) % 1
local onBeat = beatPhase < 0.1
```

### Sample Rate and Frequency Ranges
- TIC-80 sample rate: 44100 Hz
- Nyquist frequency: 22050 Hz  
- Both FFT and CQT analyze 0-22050 Hz
- CQT specifically tuned for 20 Hz - 20480 Hz (musical range)

## CQT Enhancement Plan: Making Notes "Pop" for Electronic Music

### Overview
Enhance the existing CQT implementation with signal processing techniques specifically designed to make musical notes stand out clearly in electronic music visualizations. This addresses the current issue where drums, noise, and overlapping harmonics can obscure the melodic content.

### Core Enhancements

#### 1. Harmonic-Percussive Separation (HPS)
- **Purpose**: Isolate harmonic (tonal) content from percussive (drums, transients)
- **Method**: Median filtering on magnitude spectrogram
  - Horizontal median filter → Enhances harmonic (stable over time)
  - Vertical median filter → Enhances percussive (stable over frequency)
- **Implementation**:
  - Store 5-7 frames of CQT history (circular buffer)
  - Apply median filters to create harmonic/percussive masks
  - Process only harmonic component through CQT display

#### 2. Spectral Whitening
- **Purpose**: Normalize the natural 1/f spectral tilt in music
- **Method**: Per-bin normalization based on long-term average
- **Implementation**:
  - Track running average per CQT bin (slow adaptation ~1-2 seconds)
  - Divide current magnitude by average (with floor to prevent divide-by-zero)
  - Optional: Use equal-loudness curves for perceptual weighting

#### 3. Variable-Q Transform with Aggressive Bass Q
- **Purpose**: Increase frequency resolution in bass region where electronic music needs it most
- **Method**: Adaptive Q factor that's much higher for low frequencies
- **Implementation**:
  - Q = 34 for 20-80 Hz (double the standard Q)
  - Q = 17 for 80-200 Hz (standard)
  - Q = 12 for 200+ Hz (slightly reduced for smoother visuals)
  - Regenerate kernels with frequency-dependent Q
  - May need 16K FFT to accommodate longer windows

#### 4. Adaptive Thresholding
- **Purpose**: Remove noise floor that varies across spectrum
- **Method**: Dynamic threshold per bin based on recent minimum
- **Implementation**:
  - Track minimum value per bin over ~1 second window
  - Set threshold at minimum + margin (e.g., 3-6 dB)
  - Zero out values below threshold

#### 5. Note Onset Enhancement
- **Purpose**: Make note attacks more visible
- **Method**: Detect rapid energy increases per bin
- **Implementation**:
  - Track rate of change per bin
  - Boost bins with positive derivatives (onset)
  - Quick attack, slow decay envelope

### Implementation Architecture

#### New Data Structures (cqtdata.h additions)
```c
// Enhancement data structures
typedef struct {
    // Variable-Q Transform
    float variableQ[CQT_BINS];                      // Q factor per bin
    bool kernelsNeedRegeneration;                   // Flag for kernel update
    
    // Harmonic-Percussive Separation
    float cqtHistory[CQT_HISTORY_SIZE][CQT_BINS];  // Circular buffer
    int historyIndex;                               // Current position
    float harmonicMask[CQT_BINS];                  // Harmonic component
    float percussiveMask[CQT_BINS];                // Percussive component
    
    // Spectral Whitening
    float binAverages[CQT_BINS];                   // Long-term averages
    float averageDecay;                             // Averaging factor (0.99)
    
    // Adaptive Thresholding  
    float noiseFloor[CQT_BINS];                    // Per-bin noise estimates
    float thresholdMargin;                          // dB above noise (3-6)
    
    // Onset Detection
    float previousMagnitudes[CQT_BINS];             // For derivative
    float onsetStrength[CQT_BINS];                 // Onset envelope
    float onsetDecay;                               // Envelope decay (0.9)
    
    // Enhanced output
    float cqtEnhanced[CQT_BINS];                   // Final enhanced data
} CqtEnhancementData;
```

#### Processing Pipeline (cqt.c modifications)
1. **Variable-Q CQT computation**:
   - Regenerate kernels if Q values changed
   - Use frequency-dependent Q factors
2. **Store in history buffer** (for HPS)
3. **Harmonic-Percussive Separation**:
   - Compute median filters on history
   - Extract harmonic component
4. **Spectral Whitening**:
   - Update running averages
   - Apply normalization
5. **Adaptive Thresholding**:
   - Update noise floor estimates
   - Apply thresholding
6. **Onset Detection**:
   - Calculate derivatives
   - Update onset envelopes
7. **Combine and output**

### API Extensions

#### New Functions
```lua
-- Enhanced CQT functions
value = cqte(bin)        -- Get enhanced CQT (with all processing)
value = cqtes(bin)       -- Get enhanced + smoothed CQT

-- Configuration functions
cqt_enhance(enable)      -- Enable/disable enhancement (default: true)
cqt_variable_q(enable)   -- Toggle variable-Q mode
cqt_bass_q(q_factor)     -- Set Q for bass region (20-80 Hz, default: 34)
cqt_hps(enable)          -- Toggle harmonic-percussive separation
cqt_whitening(enable)    -- Toggle spectral whitening
cqt_threshold(margin)    -- Set noise threshold margin (0-10 dB)
cqt_onset_boost(factor)  -- Set onset enhancement (0-2.0)
```

### Performance Considerations

#### Computational Cost
- **HPS**: ~0.5ms for median filtering (acceptable)
- **Whitening**: Negligible (simple division)
- **Thresholding**: Negligible (comparison)
- **Onset**: Negligible (subtraction + envelope)
- **Total overhead**: ~0.5-1ms additional

#### Memory Usage
- History buffer: 5 frames × 120 bins × 4 bytes = 2.4KB
- Enhancement data: ~3KB total
- Still well within fantasy computer constraints

### Configuration Options

#### Tunable Parameters
- **HPS window**: 5-7 frames (time) × 3-5 bins (frequency)
- **Whitening time constant**: 0.98-0.995 (1-2 second adaptation)
- **Threshold margin**: 3-6 dB above noise floor
- **Onset boost**: 1.5-3.0x multiplier
- **Enhancement mix**: 0-100% enhanced vs raw

### Testing Strategy

#### Test Scenarios
1. **Electronic track with heavy bass**: Should isolate bass notes from kick
2. **Chord progressions**: Should show clear note changes
3. **Melody over drums**: Should suppress drum interference
4. **Ambient/noise**: Should adapt to varying noise floor
5. **Fast arpeggios**: Should highlight note onsets

#### Visualization Modes
- Side-by-side comparison (raw vs enhanced)
- Individual enhancement layers (harmonic, percussive, etc.)
- Onset detection visualization
- Noise floor tracking display

### Implementation Phases

#### Phase 1: Variable-Q Transform (Foundation)
- Modify kernel generation to accept per-bin Q values
- Implement aggressive Q for bass frequencies (20-80 Hz)
- Test frequency resolution improvements
- May require switching to 16K FFT for bass accuracy

#### Phase 2: Spectral Whitening (Simplest enhancement)
- Add running average tracking
- Implement normalization
- Test with electronic music

#### Phase 3: Adaptive Thresholding
- Add noise floor estimation
- Implement thresholding
- Combine with whitening

#### Phase 4: Harmonic-Percussive Separation
- Add history buffer
- Implement median filters
- Test separation quality

#### Phase 5: Onset Enhancement
- Add derivative calculation
- Implement onset envelopes
- Fine-tune parameters

#### Phase 6: API and Integration
- Add Lua API functions
- Create demo visualizations
- Document usage

### Expected Results

For electronic music visualization:
- **Before**: All CQT bins active, drums obscure notes
- **After**: Only active musical notes visible, clean separation
- **Visual impact**: Notes "pop" with clear onset and sustain
- **Frame rate**: Minimal impact (still ~5 fps with 8K FFT)

### Future Extensions

1. **Genre-specific presets**: EDM, ambient, classical settings
2. **MIDI output**: Convert enhanced CQT to MIDI notes
3. **Key detection**: Analyze enhanced harmonic content
4. **Chord recognition**: Pattern matching on clean note data
5. **Multi-resolution**: Different processing for bass/treble

## Detailed Variable-Q Implementation Plan

### Overview
Variable-Q CQT allows different frequency resolution across the spectrum, optimized for electronic music where bass note separation is critical. With 16K FFT, we can achieve excellent bass resolution while maintaining computational efficiency.

### Q Factor Design

#### Frequency-Dependent Q Values
```
20-40 Hz:    Q = 34   (2.9% bandwidth, ~1 semitone)
40-80 Hz:    Q = 28   (3.6% bandwidth, ~0.6 semitones)
80-160 Hz:   Q = 20   (5% bandwidth, ~0.8 semitones)
160-320 Hz:  Q = 17   (5.9% bandwidth, standard CQT)
320-640 Hz:  Q = 14   (7.1% bandwidth, slightly wider)
640+ Hz:     Q = 12   (8.3% bandwidth, smoother visualization)
```

#### Rationale
- **Ultra-high Q in sub-bass** (20-40 Hz): Electronic music often has closely spaced bass notes
- **High Q in bass** (40-80 Hz): Critical for distinguishing kick from bass
- **Gradual reduction**: Smoother visualization at higher frequencies where exact pitch less critical

### Implementation Details

#### 1. Kernel Generation Modifications (cqt_kernel.c)

##### Calculate Variable Q
```c
float calculateVariableQ(float centerFreq) {
    if (centerFreq < 40.0f) return 34.0f;
    else if (centerFreq < 80.0f) return 28.0f;
    else if (centerFreq < 160.0f) return 20.0f;
    else if (centerFreq < 320.0f) return 17.0f;
    else if (centerFreq < 640.0f) return 14.0f;
    else return 12.0f;
}
```

##### Window Length Calculation
```c
// In generateSingleKernel()
float Q = calculateVariableQ(centerFreq);
int windowLength = (int)(Q * sampleRate / centerFreq);

// With 16K FFT, we can accommodate much longer windows
if (windowLength > fftSize) {
    // For very low frequencies, apply gentle tapering instead of hard truncation
    windowLength = fftSize;
    // Adjust Q to match actual window: Q_effective = windowLength * centerFreq / sampleRate
}
```

##### Expected Window Lengths (16K FFT)
- 20 Hz: Q=34 → 74,970 samples → clamped to 16,384 (Q_eff ≈ 7.4)
- 30 Hz: Q=34 → 49,980 samples → clamped to 16,384 (Q_eff ≈ 11.2)
- 40 Hz: Q=28 → 30,975 samples → clamped to 16,384 (Q_eff ≈ 14.9)
- 60 Hz: Q=28 → 20,650 samples → clamped to 16,384 (Q_eff ≈ 22.3)
- 80 Hz: Q=20 → 11,025 samples → fits! (Q = 20)
- 100+ Hz: All fit within 16K samples

#### 2. Memory Management

##### Sparse Kernel Storage Adaptation
With variable Q, kernel sparsity varies by frequency:
- Low frequencies (high Q): More non-zero values
- High frequencies (low Q): Fewer non-zero values

```c
// Adaptive sparsity threshold
float getSparsityThreshold(float centerFreq) {
    float Q = calculateVariableQ(centerFreq);
    // Higher Q needs lower threshold to preserve frequency selectivity
    if (Q > 30) return 0.005f;
    else if (Q > 20) return 0.01f;
    else return 0.02f;
}
```

#### 3. Kernel Normalization

Variable-Q requires careful normalization to ensure consistent output levels:

```c
// Energy normalization per kernel
float kernelEnergy = 0.0f;
for (int i = 0; i < windowLength; i++) {
    kernelEnergy += window[i] * window[i];
}
float normFactor = sqrtf(windowLength / kernelEnergy);

// Apply to kernel after windowing
for (int i = 0; i < windowLength; i++) {
    kernel[i] *= normFactor;
}
```

### 16K FFT Configuration

#### Update cqtdata.h
```c
// Change from 8K to 16K
#define CQT_FFT_SIZE 16384

// Add variable-Q configuration
#define CQT_VARIABLE_Q_ENABLED 1
#define CQT_BASS_Q_FACTOR 34.0f
#define CQT_MID_Q_FACTOR 17.0f
#define CQT_TREBLE_Q_FACTOR 12.0f
```

#### Update Buffer Management (fft.c)
```c
// AUDIO_BUFFER_SIZE will automatically adjust to 16384
// This provides 371ms of audio at 44.1kHz
// Update rate: 44100/16384 = 2.69 fps
```

### Performance Optimization

#### 1. Kernel Caching Strategy
Since kernels are larger with variable-Q:
- Pre-compute all kernels at startup
- Store in optimized sparse format
- Total memory: ~400-500KB (acceptable)

#### 2. Processing Optimization
```c
// Process CQT every N frames if needed
static int frameCounter = 0;
if (++frameCounter >= CQT_PROCESS_INTERVAL) {
    frameCounter = 0;
    CQT_Process(audioBuffer, fftBuffer, cqtData);
}
```

#### 3. SIMD Considerations
- Ensure kernel application loops are vectorization-friendly
- Keep data aligned for SIMD operations
- Profile on target platforms

### Testing and Validation

#### Test Signal Generation
Create test signals for each Q region:
```lua
-- Test script for variable-Q validation
function generateTestTone(freq, duration)
    -- Generate pure tone at specific frequency
    -- Measure CQT bin spread
    -- Verify Q factor matches design
end

-- Test cases:
-- 25 Hz: Should show narrow peak (Q=34)
-- 50 Hz: Should show narrow peak (Q=28)
-- 100 Hz: Should show moderate peak (Q=20)
-- 440 Hz: Should show standard CQT peak (Q=14)
```

#### Measurement Metrics
1. **3dB Bandwidth**: Measure actual vs theoretical
2. **Sidelobe Suppression**: Should be >40dB
3. **Cross-talk**: Adjacent bins should have <-20dB leakage

### Integration with Enhancement Pipeline

#### Variable-Q as Foundation
Variable-Q must be implemented first because:
1. Kernel generation is fundamental to CQT
2. Other enhancements depend on accurate frequency detection
3. Memory layout changes affect all subsequent processing

#### Interaction with Other Enhancements
- **HPS**: Benefits from better frequency resolution
- **Whitening**: May need per-Q normalization
- **Thresholding**: Noise floor varies with Q
- **Onset**: Higher Q means better temporal smearing

### API Implementation

#### Configuration Functions
```c
// In cqt.c
static float bassQFactor = 34.0f;
static float midQFactor = 17.0f;
static float trebleQFactor = 12.0f;
static bool variableQEnabled = true;

void CQT_SetVariableQ(bool enabled) {
    if (variableQEnabled != enabled) {
        variableQEnabled = enabled;
        CQT_RegenerateKernels();
    }
}

void CQT_SetBassQ(float q) {
    if (bassQFactor != q) {
        bassQFactor = q;
        if (variableQEnabled) CQT_RegenerateKernels();
    }
}
```

### Expected Results with 16K FFT

#### Bass Region (20-80 Hz)
- **20 Hz**: Q_eff ≈ 7.4 (limited by FFT size, but much better than current 3.7)
- **30 Hz**: Q_eff ≈ 11.2 (good separation)
- **40 Hz**: Q_eff ≈ 14.9 (excellent)
- **60 Hz**: Q_eff = 22.3 (better than designed!)
- **80 Hz**: Q = 20 (perfect)

#### Electronic Music Benefits
1. **Sub-bass**: Can distinguish notes 1-2 semitones apart
2. **Bass**: Clear separation between kick and bassline
3. **Midrange**: Standard CQT resolution
4. **Treble**: Smooth visualization without artifacts

### Migration Path

#### From 8K to 16K FFT
1. Update `CQT_FFT_SIZE` in cqtdata.h
2. Verify buffer allocation in fft.c
3. Test performance on target platforms
4. Adjust frame processing if needed

#### Backwards Compatibility
- Keep 8K as compile-time option
- Allow runtime FFT size selection (future)
- Maintain existing API behavior

### Debugging and Profiling

#### Debug Output
```c
// Add debug info for each kernel
printf("Bin %d: Freq %.1f Hz, Q=%.1f, Window=%d samples, Q_eff=%.1f\n",
       bin, centerFreq, Q, windowLength, effectiveQ);
```

#### Performance Profiling
- Measure kernel generation time
- Track per-frame CQT processing time
- Monitor memory usage
- Test on various CPUs

### Future Enhancements

1. **Smooth Q Transitions**: Interpolate Q between frequency bands
2. **Adaptive Q**: Adjust based on signal content
3. **Multi-resolution**: Different FFT sizes for different octaves
4. **GPU Acceleration**: Parallel kernel application

## Important Notes

- The project supports multiple scripting languages, all exposed through the same API
- Platform-specific code should be isolated in src/system/
- The studio uses immediate mode GUI principles
- Cartridge format is documented in wiki and src/studio/project.c
- PRO version enables additional features like extra memory banks
