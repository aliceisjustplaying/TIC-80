#include "cqt.h"
#include "cqt_kernel.h"
#include "../cqtdata.h"
#include "../fftdata.h"
#include "kiss_fftr.h"
#include <math.h>
#include <string.h>

// FFT configuration for CQT
static kiss_fftr_cfg cqtFftCfg = NULL;
static float* cqtAudioBuffer = NULL;
static kiss_fft_cpx* cqtFftOutput = NULL;

// Initialize CQT processing
bool CQT_Open(void)
{
    // Initialize data structures
    CQT_Init();
    
    // Allocate FFT buffers
    cqtAudioBuffer = (float*)malloc(CQT_FFT_SIZE * sizeof(float));
    cqtFftOutput = (kiss_fft_cpx*)malloc((CQT_FFT_SIZE/2 + 1) * sizeof(kiss_fft_cpx));
    
    if (!cqtAudioBuffer || !cqtFftOutput)
    {
        CQT_Close();
        return false;
    }
    
    // Create FFT configuration
    cqtFftCfg = kiss_fftr_alloc(CQT_FFT_SIZE, 0, NULL, NULL);
    if (!cqtFftCfg)
    {
        CQT_Close();
        return false;
    }
    
    // Configure CQT kernels
    CqtKernelConfig config = {
        .fftSize = CQT_FFT_SIZE,
        .numBins = CQT_BINS,
        .minFreq = CQT_MIN_FREQ,
        .maxFreq = CQT_MAX_FREQ,
        .sampleRate = 44100.0f,  // Standard TIC-80 sample rate
        .windowType = CQT_WINDOW_HAMMING,
        .sparsityThreshold = CQT_SPARSITY_THRESHOLD
    };
    
    // Generate kernels
    if (!CQT_GenerateKernels(cqtKernels, &config))
    {
        CQT_Close();
        return false;
    }
    
    return true;
}

// Apply CQT kernels to FFT output
void CQT_ApplyKernels(const float* fftReal, const float* fftImag)
{
    // Clear CQT output
    memset(cqtData, 0, sizeof(cqtData));
    
    // Apply each kernel to compute CQT bins
    for (int bin = 0; bin < CQT_BINS; bin++)
    {
        CqtKernel* kernel = &cqtKernels[bin];
        float real = 0.0f;
        float imag = 0.0f;
        
        // Sparse matrix multiplication
        for (int k = 0; k < kernel->length; k++)
        {
            int idx = kernel->indices[k];
            // Complex multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
            real += fftReal[idx] * kernel->real[k] - fftImag[idx] * kernel->imag[k];
            imag += fftReal[idx] * kernel->imag[k] + fftImag[idx] * kernel->real[k];
        }
        
        // Calculate magnitude
        cqtData[bin] = sqrt(real * real + imag * imag);
    }
}

// Process CQT from audio data (using existing FFT data from fftdata.h)
void CQT_Process(const float* fftReal, const float* fftImag)
{
    if (!cqtFftCfg || !cqtEnabled) return;
    
    // Note: In the full implementation, we would:
    // 1. Get audio data from the shared buffer
    // 2. Perform our own 4096-point FFT
    // For now, we'll process using provided FFT data
    
    // Apply CQT kernels
    CQT_ApplyKernels(fftReal, fftImag);
    
    // Apply smoothing
    for (int i = 0; i < CQT_BINS; i++)
    {
        cqtSmoothingData[i] = cqtSmoothingData[i] * CQT_SMOOTHING_FACTOR + 
                              cqtData[i] * (1.0f - CQT_SMOOTHING_FACTOR);
    }
    
    // Find peak for normalization
    float currentPeak = 0.0f;
    for (int i = 0; i < CQT_BINS; i++)
    {
        if (cqtSmoothingData[i] > currentPeak)
            currentPeak = cqtSmoothingData[i];
    }
    
    // Smooth peak value
    if (currentPeak > cqtPeakSmoothValue)
        cqtPeakSmoothValue = currentPeak;
    else
        cqtPeakSmoothValue = cqtPeakSmoothValue * 0.99f + currentPeak * 0.01f;
    
    // Normalize data
    float normalizer = (cqtPeakSmoothValue > 0.0001f) ? (1.0f / cqtPeakSmoothValue) : 1.0f;
    for (int i = 0; i < CQT_BINS; i++)
    {
        cqtNormalizedData[i] = cqtSmoothingData[i] * normalizer;
        if (cqtNormalizedData[i] > 1.0f) cqtNormalizedData[i] = 1.0f;
    }
}

// Close CQT processing and free resources
void CQT_Close(void)
{
    if (cqtFftCfg)
    {
        free(cqtFftCfg);
        cqtFftCfg = NULL;
    }
    
    if (cqtAudioBuffer)
    {
        free(cqtAudioBuffer);
        cqtAudioBuffer = NULL;
    }
    
    if (cqtFftOutput)
    {
        free(cqtFftOutput);
        cqtFftOutput = NULL;
    }
    
    // Clean up kernels
    CQT_Cleanup();
}