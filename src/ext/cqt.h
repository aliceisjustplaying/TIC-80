#pragma once

#include <stdbool.h>

// Initialize CQT processing
// Returns true on success, false on failure
bool CQT_Open(void);

// Process CQT from audio buffer (uses shared audio capture buffer)
void CQT_ProcessAudio(void);

// Close CQT processing and free resources
void CQT_Close(void);

// Apply CQT kernels to FFT output
void CQT_ApplyKernels(const float* fftReal, const float* fftImag);