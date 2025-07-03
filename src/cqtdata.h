#pragma once
#include <stdbool.h>

#define CQT_BINS 120
#define CQT_OCTAVES 10
#define CQT_BINS_PER_OCTAVE 12
#define CQT_FFT_SIZE 4096  // Larger FFT for better sub-bass resolution

// CQT frequency range
#define CQT_MIN_FREQ 20.0f    // Sub-bass for electronic music
#define CQT_MAX_FREQ 20480.0f // Nearest note to 20kHz

// Smoothing parameters
#define CQT_SMOOTHING_FACTOR 0.3f  // Reduced from 0.7f for more responsive display
#define CQT_SPARSITY_THRESHOLD 0.01f

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