#include "cqtdata.h"
#include <string.h>
#include <stdlib.h>

// Global CQT data arrays
float cqtData[CQT_BINS];
float cqtSmoothingData[CQT_BINS];
float cqtNormalizedData[CQT_BINS];

// Peak tracking for auto-gain
float cqtPeakValue = 1.0f;
float cqtPeakSmoothValue = 1.0f;

// Enable flag (tied to fftEnabled initially)
bool cqtEnabled = false;

// Array of kernels, one per CQT bin
CqtKernel cqtKernels[CQT_BINS];

void CQT_Init(void)
{
    // Zero all data arrays
    memset(cqtData, 0, sizeof(cqtData));
    memset(cqtSmoothingData, 0, sizeof(cqtSmoothingData));
    memset(cqtNormalizedData, 0, sizeof(cqtNormalizedData));
    
    // Initialize peak values
    cqtPeakValue = 1.0f;
    cqtPeakSmoothValue = 1.0f;
    
    // Zero kernel pointers
    memset(cqtKernels, 0, sizeof(cqtKernels));
}

void CQT_Cleanup(void)
{
    // Free all allocated kernel memory
    for (int i = 0; i < CQT_BINS; i++)
    {
        if (cqtKernels[i].real)
        {
            free(cqtKernels[i].real);
            cqtKernels[i].real = NULL;
        }
        if (cqtKernels[i].imag)
        {
            free(cqtKernels[i].imag);
            cqtKernels[i].imag = NULL;
        }
        if (cqtKernels[i].indices)
        {
            free(cqtKernels[i].indices);
            cqtKernels[i].indices = NULL;
        }
        cqtKernels[i].length = 0;
    }
}