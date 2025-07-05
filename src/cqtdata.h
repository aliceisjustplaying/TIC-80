#pragma once
#include <stdbool.h>

#define CQT_BINS 120
#define CQT_OCTAVES 10
#define CQT_BINS_PER_OCTAVE 12
#define CQT_FFT_SIZE 8192   // 8K FFT - optimized variable-Q for responsive visualization, ~5.4 fps

// CQT frequency range
#define CQT_MIN_FREQ 20.0f    // Sub-bass for electronic music
#define CQT_MAX_FREQ 20480.0f // Nearest note to 20kHz

// Smoothing parameters
#define CQT_SMOOTHING_FACTOR 0.3f  // Reduced from 0.7f for more responsive display
#define CQT_SPARSITY_THRESHOLD 0.01f

// Variable-Q configuration (optimized for 8K FFT)
#define CQT_VARIABLE_Q_ENABLED 1
#define CQT_8K_OPTIMIZED 1          // Uses Q values that fit within 8K FFT
#define CQT_BASS_Q_MIN 7.4f         // Minimum Q at 20 Hz (constrained by 8K)
#define CQT_BASS_Q_MAX 17.0f        // Full Q achieved at 80+ Hz
#define CQT_TREBLE_Q_FACTOR 11.0f   // Smoother for high frequencies

// Raw CQT magnitude data
extern float cqtData[CQT_BINS];

// Smoothed CQT data for visual stability
extern float cqtSmoothingData[CQT_BINS];

// Normalized CQT data (0-1 range)
extern float cqtNormalizedData[CQT_BINS];

// Peak tracking for auto-gain
extern float cqtPeakValue;
extern float cqtPeakSmoothValue;

// Enable flag (tied to fftEnabled initially)
extern bool cqtEnabled;

// Sparse kernel storage structures
typedef struct {
    float* real;      // Real parts of kernel (sparse)
    float* imag;      // Imaginary parts of kernel (sparse)
    int* indices;     // Non-zero FFT bin indices
    int length;       // Number of non-zero elements
} CqtKernel;

// Array of kernels, one per CQT bin
extern CqtKernel cqtKernels[CQT_BINS];

// Initialize CQT data structures
void CQT_Init(void);

// Free CQT kernel memory
void CQT_Cleanup(void);