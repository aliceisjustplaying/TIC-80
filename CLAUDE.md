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

## Audio Processing Features

### FFT Implementation

TIC-80 includes FFT support for audio visualization and livecoding.

#### Architecture
- **src/ext/fft.c**: Audio capture and FFT processing using KISS FFT
- **src/fftdata.h/c**: FFT data storage (1024 bins, smoothing, normalization)
- **Processing**: In `tic_core_tick` before script execution

#### Specifications
- Sample rate: 44100 Hz
- Window size: 2048 samples (46ms)
- Frequency bins: 1024 (0-22050 Hz, ~21.5 Hz per bin)
- Update rate: ~21 fps
- Smoothing factor: 0.6
- Auto-gain control with peak detection

#### API Functions
```lua
value = fft(bin)   -- Get raw FFT magnitude for bin (0-1023)
value = ffts(bin)  -- Get smoothed FFT magnitude for bin (0-1023)
```

### CQT Implementation

Constant-Q Transform provides logarithmic frequency spacing for better musical analysis.

#### Architecture
- **src/ext/cqt.c**: CQT processing with separate FFT
- **src/ext/cqt_kernel.c**: Kernel generation with sparse storage
- **src/cqtdata.h/c**: CQT data storage and configuration
- **Processing**: In `tic_core_tick` after FFT processing

#### Specifications
- Frequency range: 20 Hz - 20480 Hz (10 octaves × 12 notes = 120 bins)
- FFT size: 8192 samples (configurable)
- Update rate: ~5.4 fps with 8K FFT
- Variable-Q implementation optimized for 8K FFT constraint
- Smoothing factor: 0.3
- Spectral whitening: Enabled by default (toggle via `CQT_SPECTRAL_WHITENING_ENABLED`)

#### Variable-Q Design (8K FFT Optimized)
| Frequency Range | Design Q | Effective Q | Resolution |
|----------------|----------|-------------|------------|
| 20-25 Hz | 7.4 | 3.7 | ~5.4 Hz |
| 25-30 Hz | 9.2 | 4.6 | ~5.4 Hz |
| 30-40 Hz | 11.5 | 5.6-7.4 | ~5.1 Hz |
| 40-50 Hz | 14.5 | 7.4-9.3 | ~4.3 Hz |
| 50-65 Hz | 16.0 | 9.3-12.1 | ~4.1 Hz |
| 65-80 Hz | 17.0 | 12.1-17.0 | ~4.7 Hz |
| 80+ Hz | 17.0 | 17.0 | Standard CQT |

#### API Functions
```lua
value = cqt(bin)   -- Get raw CQT magnitude for bin (0-119)
-- Note mapping: Bin = octave * 12 + note
-- Note: C=0, C#=1, D=2, D#=3, E=4, F=5, F#=6, G=7, G#=8, A=9, A#=10, B=11
```

### FFT vs CQT Comparison

| Aspect | FFT | CQT |
|--------|-----|-----|
| Frequency spacing | Linear (21.5 Hz/bin) | Logarithmic (musical) |
| Update rate | ~21 fps | ~5.4 fps |
| Window size | 2048 samples (46ms) | Variable (up to 8192) |
| Best for | Beat detection, rhythm | Note detection, harmony |
| Low freq resolution | ±10.75 Hz | ~5 Hz @ 20 Hz |
| High freq resolution | ±10.75 Hz | Proportional to frequency |

### Shared Audio Buffer Architecture

Both FFT and CQT share the same audio capture buffer:
- Buffer size: Maximum of (2048, CQT_FFT_SIZE) samples
- FFT reads samples 0-2047
- CQT reads samples 0-(CQT_FFT_SIZE-1)
- Uses miniaudio for audio capture (mic or loopback on Windows)

### Practical Usage Guidelines

**Use FFT for:**
- Kick drum detection (bins 2-4, ~40-80 Hz)
- Beat synchronization
- Energy meters by frequency band
- Any rhythm visualization above 150 BPM

**Use CQT for:**
- Bass note identification
- Chord/key detection
- Color mapping from musical content
- Melodic visualization

**Hybrid Approach Example:**
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
```

## Current Implementation Status

### Completed Features
- **FFT**: 1024 bins with exact original behavior preserved
- **CQT**: 120 bins with Variable-Q implementation
- **Spectral Whitening**: Per-bin normalization for CQT
- **Shared Audio Buffer**: Automatic sizing for both FFT and CQT
- **Lua API**: `fft()`, `ffts()`, `cqt()` functions implemented

### Configuration Options
- `CQT_FFT_SIZE`: Default 8192 (configurable in cqtdata.h)
- `CQT_SPECTRAL_WHITENING_ENABLED`: Toggle spectral whitening (0/1)
- `CQT_WHITENING_DECAY`: Running average decay factor (default 0.99)
- `CQT_SMOOTHING_FACTOR`: CQT smoothing factor (default 0.3)

### Test Scripts
- `demo_fft_cqt_hybrid.lua`: Combined FFT/CQT visualization
- `test_cqt_spectrum_v2.lua`: CQT spectrum analyzer
- `test_fft_restored.lua`: FFT verification
- `test_cqt_variable_q.lua`: Variable-Q demonstration
- `test_cqt_whitening.lua`: Spectral whitening comparison

## Future Enhancements

### Additional API Functions
- `cqts(bin)`: Smoothed CQT data
- `cqto(octave, note)`: Raw CQT by musical note
- `cqtos(octave, note)`: Smoothed CQT by musical note

### Signal Processing Enhancements
- **Harmonic-Percussive Separation**: Isolate tonal content from drums
- **Adaptive Thresholding**: Dynamic noise floor removal
- **Note Onset Enhancement**: Make note attacks more visible
- **Enhanced Variable-Q**: Support for 16K FFT for better bass resolution

### Platform Support
- Add CQT to other language bindings (currently Lua only)
- GPU acceleration for kernel application
- Configurable FFT sizes at runtime

## Important Notes

- The project supports multiple scripting languages, all exposed through the same API
- Platform-specific code should be isolated in src/system/
- The studio uses immediate mode GUI principles
- Cartridge format is documented in wiki and src/studio/project.c
- PRO version enables additional features like extra memory banks

## Harmonic-Percussive Separation (HPS) Plan

### Overview

Harmonic-Percussive Separation isolates tonal (harmonic) content from rhythmic (percussive) content in audio signals. This enhancement will apply HPS exclusively to CQT data, providing better separation of musical elements like sustained notes from drum hits.

### Algorithm Design

**Median Filtering Approach (Fitzgerald 2010):**
- Apply median filters on CQT magnitude spectrogram
- Horizontal (time) median filter → captures harmonic content (sustained tones)
- Vertical (frequency) median filter → captures percussive content (transients)
- Generate masks by comparing filtered outputs
- Apply masks to separate harmonic and percussive components

### Architecture

#### New Files
- **src/ext/hps.c**: HPS processing implementation
- **src/ext/hps.h**: HPS interface and function declarations
- **src/hpsdata.h/c**: HPS data structures and storage

#### Data Flow
```
Audio Buffer → CQT Processing → CQT Magnitude → HPS Processing → Separated Components
                                                       ↓
                                              Harmonic & Percussive CQT
```

#### Integration in tic_core_tick
```c
if (fftEnabled) {
    FFT_GetFFT(fftData);  // Regular FFT processing (unchanged)
    
    if (cqtEnabled) {
        CQT_ProcessAudio();
        
        if (hpsEnabled) {
            HPS_ProcessCQT(cqtData);  // Apply HPS to CQT only
        }
    }
}
```

### Data Structures

```c
// In hpsdata.h
#define HPS_HISTORY_SIZE 32      // Frames of history for median filtering
#define HPS_MEDIAN_SIZE_H 17     // Horizontal filter size (0.8-1.5s @ 5.4fps)
#define HPS_MEDIAN_SIZE_P 17     // Vertical filter size (1.4 octaves)

typedef struct {
    // CQT magnitude history buffer
    float cqtHistory[HPS_HISTORY_SIZE][120];
    int historyIndex;
    
    // Separated CQT components
    float harmonicCQT[120];      // Sustained tonal content
    float percussiveCQT[120];    // Transient/rhythmic content
    
    // Smoothed outputs
    float harmonicSmoothing[120];
    float percussiveSmoothing[120];
    
    // Normalization data
    float harmonicNormalized[120];
    float percussiveNormalized[120];
    
    // Configuration
    float separationStrength;    // 0.0-1.0, controls mask hardness
    float harmonicGain;         // Gain for harmonic component
    float percussiveGain;       // Gain for percussive component
    bool enabled;
} HpsData;
```

### API Functions

```lua
-- CQT-based HPS functions (update rate ~5.4 fps)
hpsh(bin)    -- Get harmonic component at CQT bin (0-119)
hpsp(bin)    -- Get percussive component at CQT bin (0-119)
hpshs(bin)   -- Get smoothed harmonic component
hpsps(bin)   -- Get smoothed percussive component

-- Musical note access (octave 0-9, note 0-11)
hpsho(octave, note)  -- Harmonic by musical note
hpspo(octave, note)  -- Percussive by musical note

-- Configuration
hpscfg(param, value) -- Configure HPS parameters
-- param: "strength" (0.0-1.0), "hgain" (0.0-2.0), "pgain" (0.0-2.0)
```

### Implementation Details

#### Median Filtering
- Use efficient sliding window median algorithm
- Handle CQT's logarithmic frequency spacing
- Circular buffer for time history
- Optimize for 120 bins and 32 frame history

#### Masking Strategy
1. Compute harmonic emphasis: `H = median_h(|CQT|²)`
2. Compute percussive emphasis: `P = median_p(|CQT|²)`
3. Generate binary masks:
   - Harmonic mask: `M_h = H >= P`
   - Percussive mask: `M_p = P > H`
4. Optional soft masking using Wiener filtering

#### Memory Usage
- History buffer: 32 × 120 × 4 bytes = 15KB
- Processing buffers: ~10KB
- Total additional memory: ~25KB

### Performance Considerations

- Process only when CQT is updated (~5.4 fps)
- Reuse existing CQT magnitude calculations
- Optimize median filters for small kernel sizes
- Cache-friendly memory access patterns
- Optional SIMD for median operations

### Example Usage

```lua
-- Visualize harmonic vs percussive content
function TIC()
    cls(0)
    
    -- Draw harmonic content (sustained notes)
    for oct=2,6 do
        for note=0,11 do
            local h = hpsh(oct*12+note) * 50
            local x = note * 20
            local y = 100 - oct * 15
            rect(x, y-h, 18, h, note+1)
        end
    end
    
    -- Draw percussive hits
    for i=0,119 do
        local p = hpsp(i)
        if p > 0.1 then
            local x = (i % 12) * 20
            local y = 120 - (i // 12) * 10
            circ(x+9, y, p*20, 15)
        end
    end
end

-- React to bass drum vs bass note
function TIC()
    local bassDrum = 0
    local bassNote = 0
    
    -- Sum percussive energy in bass range (20-80 Hz)
    for i=0,35 do  -- First 3 octaves
        bassDrum = bassDrum + hpsp(i)
    end
    
    -- Find strongest harmonic in bass range
    for i=24,35 do  -- 2nd octave
        if hpsh(i) > hpsh(bassNote) then
            bassNote = i
        end
    end
    
    -- Visualize separately
    cls(0)
    circ(120, 68, bassDrum * 30, 8)  -- Drum pulse
    print("Note: " .. (bassNote % 12), 100, 60, (bassNote % 12) + 1)
end
```

### Implementation Phases

**Phase 1: Core HPS Algorithm**
- Implement circular buffer for CQT history
- Create median filtering functions
- Implement basic binary masking
- Add core API functions

**Phase 2: Enhancements**
- Add Wiener soft masking option
- Implement smoothing and normalization
- Add gain controls
- Optimize performance

**Phase 3: Extended API**
- Add musical note access functions
- Implement configuration system
- Create demo scripts
- Documentation

### Benefits

- **Better Bass Separation**: Distinguish bass drums from bass notes
- **Cleaner Visualizations**: Separate melody from rhythm
- **Musical Analysis**: Identify chord progressions without drum interference
- **Creative Effects**: Process harmonic and percussive content differently