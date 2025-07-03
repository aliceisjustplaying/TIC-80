#pragma once

#include <stdbool.h>
#include "../cqtdata.h"

// Window types for CQT kernels
typedef enum {
    CQT_WINDOW_HAMMING = 0,
    CQT_WINDOW_GAUSSIAN
} CqtWindowType;

// CQT kernel configuration
typedef struct {
    int fftSize;              // FFT size (4096)
    int numBins;              // Number of CQT bins (120)
    float minFreq;            // Minimum frequency (20 Hz)
    float maxFreq;            // Maximum frequency (20480 Hz)
    float sampleRate;         // Sample rate (44100 Hz)
    CqtWindowType windowType; // Window function type
    float sparsityThreshold;  // Threshold for sparse matrix (0.01)
} CqtKernelConfig;

// Generate CQT kernels for all bins
// Returns true on success, false on failure
bool CQT_GenerateKernels(CqtKernel* kernels, const CqtKernelConfig* config);

// Calculate Q factor for given parameters
float CQT_CalculateQ(int binsPerOctave);

// Generate center frequencies for CQT bins
void CQT_GenerateCenterFrequencies(float* frequencies, int numBins, float minFreq, float maxFreq);