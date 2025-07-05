#include "vqtdata.h"
#include <string.h>
#include <stdlib.h>

// Global VQT data arrays
float vqtData[VQT_BINS];
float vqtSmoothingData[VQT_BINS];
float vqtNormalizedData[VQT_BINS];

// Peak tracking for auto-gain
float vqtPeakValue = 1.0f;
float vqtPeakSmoothValue = 1.0f;

// Enable flag (tied to fftEnabled initially)
bool vqtEnabled = false;

// Array of kernels, one per VQT bin
VqtKernel vqtKernels[VQT_BINS];

// Spectral whitening data
float vqtBinAverages[VQT_BINS];
float vqtWhitenedData[VQT_BINS];

void VQT_Init(void)
{
    // Zero all data arrays
    memset(vqtData, 0, sizeof(vqtData));
    memset(vqtSmoothingData, 0, sizeof(vqtSmoothingData));
    memset(vqtNormalizedData, 0, sizeof(vqtNormalizedData));
    
    // Initialize peak values
    vqtPeakValue = 1.0f;
    vqtPeakSmoothValue = 1.0f;
    
    // Zero kernel pointers
    memset(vqtKernels, 0, sizeof(vqtKernels));
    
    // Initialize spectral whitening arrays
    memset(vqtBinAverages, 0, sizeof(vqtBinAverages));
    memset(vqtWhitenedData, 0, sizeof(vqtWhitenedData));
    
    // Start with small initial averages to avoid divide-by-zero
    for (int i = 0; i < VQT_BINS; i++)
    {
        vqtBinAverages[i] = VQT_WHITENING_FLOOR;
    }
}

void VQT_Cleanup(void)
{
    // Free all allocated kernel memory
    for (int i = 0; i < VQT_BINS; i++)
    {
        if (vqtKernels[i].real)
        {
            free(vqtKernels[i].real);
            vqtKernels[i].real = NULL;
        }
        if (vqtKernels[i].imag)
        {
            free(vqtKernels[i].imag);
            vqtKernels[i].imag = NULL;
        }
        if (vqtKernels[i].indices)
        {
            free(vqtKernels[i].indices);
            vqtKernels[i].indices = NULL;
        }
        vqtKernels[i].length = 0;
    }
}