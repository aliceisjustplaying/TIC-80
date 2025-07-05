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

### VQT Implementation

Constant-Q Transform provides logarithmic frequency spacing for better musical analysis.

#### Architecture
- **src/ext/vqt.c**: VQT processing with separate FFT
- **src/ext/vqt_kernel.c**: Kernel generation with sparse storage
- **src/vqtdata.h/c**: VQT data storage and configuration
- **Processing**: In `tic_core_tick` after FFT processing

#### Specifications
- Frequency range: 20 Hz - 20480 Hz (10 octaves × 12 notes = 120 bins)
- FFT size: 8192 samples (configurable)
- Update rate: ~5.4 fps with 8K FFT
- Variable-Q implementation optimized for 8K FFT constraint
- Smoothing factor: 0.3
- Spectral whitening: Enabled by default (toggle via `VQT_SPECTRAL_WHITENING_ENABLED`)

#### Variable-Q Design (8K FFT Optimized)
| Frequency Range | Design Q | Effective Q | Resolution |
|----------------|----------|-------------|------------|
| 20-25 Hz | 7.4 | 3.7 | ~5.4 Hz |
| 25-30 Hz | 9.2 | 4.6 | ~5.4 Hz |
| 30-40 Hz | 11.5 | 5.6-7.4 | ~5.1 Hz |
| 40-50 Hz | 14.5 | 7.4-9.3 | ~4.3 Hz |
| 50-65 Hz | 16.0 | 9.3-12.1 | ~4.1 Hz |
| 65-80 Hz | 17.0 | 12.1-17.0 | ~4.7 Hz |
| 80+ Hz | 17.0 | 17.0 | Standard VQT |

#### API Functions
```lua
value = vqt(bin)   -- Get raw VQT magnitude for bin (0-119)
-- Note mapping: Bin = octave * 12 + note
-- Note: C=0, C#=1, D=2, D#=3, E=4, F=5, F#=6, G=7, G#=8, A=9, A#=10, B=11
```

### FFT vs VQT Comparison

| Aspect | FFT | VQT |
|--------|-----|-----|
| Frequency spacing | Linear (21.5 Hz/bin) | Logarithmic (musical) |
| Update rate | ~21 fps | ~5.4 fps |
| Window size | 2048 samples (46ms) | Variable (up to 8192) |
| Best for | Beat detection, rhythm | Note detection, harmony |
| Low freq resolution | ±10.75 Hz | ~5 Hz @ 20 Hz |
| High freq resolution | ±10.75 Hz | Proportional to frequency |

### Shared Audio Buffer Architecture

Both FFT and VQT share the same audio capture buffer:
- Buffer size: Maximum of (2048, VQT_FFT_SIZE) samples
- FFT reads samples 0-2047
- VQT reads samples 0-(VQT_FFT_SIZE-1)
- Uses miniaudio for audio capture (mic or loopback on Windows)

### Practical Usage Guidelines

**Use FFT for:**
- Kick drum detection (bins 2-4, ~40-80 Hz)
- Beat synchronization
- Energy meters by frequency band
- Any rhythm visualization above 150 BPM

**Use VQT for:**
- Bass note identification
- Chord/key detection
- Color mapping from musical content
- Melodic visualization

**Hybrid Approach Example:**
```lua
-- Rhythm from FFT (21 fps)
local kick = fft(2) + fft(3) + fft(4)

-- Musical content from VQT (5.4 fps)
local bassNote = 0
for i=24,35 do  -- 2nd octave
  if vqt(i) > vqt(bassNote) then bassNote = i end
end

-- Combine for visuals
local pulse = kick * 2  -- Size from rhythm
local color = (bassNote % 12) + 1  -- Color from note
```

## Current Implementation Status

### Completed Features
- **FFT**: 1024 bins with exact original behavior preserved
- **VQT**: 120 bins with Variable-Q implementation
- **Spectral Whitening**: Per-bin normalization for VQT
- **Shared Audio Buffer**: Automatic sizing for both FFT and VQT
- **Lua API**: `fft()`, `ffts()`, `vqt()` functions implemented

### Configuration Options
- `VQT_FFT_SIZE`: Default 8192 (configurable in vqtdata.h)
- `VQT_SPECTRAL_WHITENING_ENABLED`: Toggle spectral whitening (0/1)
- `VQT_WHITENING_DECAY`: Running average decay factor (default 0.99)
- `VQT_SMOOTHING_FACTOR`: VQT smoothing factor (default 0.3)

### Test Scripts
- `demo_fft_vqt_hybrid.lua`: Combined FFT/VQT visualization
- `test_vqt_spectrum_v2.lua`: VQT spectrum analyzer
- `test_fft_restored.lua`: FFT verification
- `test_vqt_variable_q.lua`: Variable-Q demonstration
- `test_vqt_whitening.lua`: Spectral whitening comparison

## Future Enhancements

### Additional API Functions
- `vqts(bin)`: Smoothed VQT data
- `vqto(octave, note)`: Raw VQT by musical note
- `vqtos(octave, note)`: Smoothed VQT by musical note

### Signal Processing Enhancements
- **Adaptive Thresholding**: Dynamic noise floor removal

### Platform Support
- Add VQT to other language bindings (currently Lua only)
- GPU acceleration for kernel application
- Configurable FFT sizes at runtime

## Important Notes

- The project supports multiple scripting languages, all exposed through the same API
- Platform-specific code should be isolated in src/system/
- The studio uses immediate mode GUI principles
- Cartridge format is documented in wiki and src/studio/project.c
- PRO version enables additional features like extra memory banks
