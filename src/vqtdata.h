#pragma once
#include <stdbool.h>

#define VQT_BINS 120
#define VQT_FFT_SIZE 8192   // 8K FFT - optimized variable-Q for responsive visualization, ~5.4 fps

// VQT frequency range
#define VQT_MIN_FREQ 20.0f    // Sub-bass for electronic music
#define VQT_MAX_FREQ 20480.0f // Nearest note to 20kHz

// Smoothing parameters
#define VQT_SMOOTHING_FACTOR 0.3f  // Reduced from 0.7f for more responsive display
#define VQT_SPARSITY_THRESHOLD 0.01f

// Raw VQT magnitude data
extern float vqtData[VQT_BINS];

// Smoothed VQT data for visual stability
extern float vqtSmoothingData[VQT_BINS];

// Normalized VQT data (0-1 range)
extern float vqtNormalizedData[VQT_BINS];

// Peak tracking for auto-gain
extern float vqtPeakSmoothValue;

// Enable flag (tied to fftEnabled initially)
extern bool vqtEnabled;

// Sparse kernel storage structures
typedef struct {
    float* real;      // Real parts of kernel (sparse)
    float* imag;      // Imaginary parts of kernel (sparse)
    int* indices;     // Non-zero FFT bin indices
    int length;       // Number of non-zero elements
} VqtKernel;

// Array of kernels, one per VQT bin
extern VqtKernel vqtKernels[VQT_BINS];

// Initialize VQT data structures
void VQT_Init(void);

// Free VQT kernel memory
void VQT_Cleanup(void);
