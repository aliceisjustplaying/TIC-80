#pragma once

#include <stdbool.h>

// Initialize CQT processing
// Returns true on success, false on failure
bool CQT_Open(void);

// Process CQT from FFT data
// fftData: Complex FFT output (size should be CQT_FFT_SIZE/2 + 1)
void CQT_Process(const float* fftReal, const float* fftImag);

// Close CQT processing and free resources
void CQT_Close(void);

// Apply CQT kernels to FFT output
void CQT_ApplyKernels(const float* fftReal, const float* fftImag);