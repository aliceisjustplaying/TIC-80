#include "cqt.h"
#include "cqt_kernel.h"
#include "../cqtdata.h"
#include "../fftdata.h"
#include "fft.h"
#include "kiss_fftr.h"
#include <math.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>

#define CQT_DEBUG

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
    
    // Debug: Print first few center frequencies and expected FFT bins
    #ifdef CQT_DEBUG
    float centerFreqs[CQT_BINS];
    CQT_GenerateCenterFrequencies(centerFreqs, CQT_BINS, CQT_MIN_FREQ, CQT_MAX_FREQ);
    printf("CQT: First 10 center frequencies:\n");
    for (int i = 0; i < 10 && i < CQT_BINS; i++)
    {
        // FFT bin = freq * fftSize / sampleRate
        int expectedBin = (int)(centerFreqs[i] * CQT_FFT_SIZE / 44100.0f);
        printf("  Bin %d: %.2f Hz -> FFT bin %d\n", i, centerFreqs[i], expectedBin);
    }
    
    // Also print expected bins for test frequencies
    printf("\nCQT: Expected bins for test frequencies:\n");
    float testFreqs[] = {110, 220, 440, 880};
    for (int i = 0; i < 4; i++)
    {
        int cqtBin = (int)(12 * log(testFreqs[i] / 20.0) / log(2.0) + 0.5);
        int fftBin = (int)(testFreqs[i] * CQT_FFT_SIZE / 44100.0f);
        printf("  %.0f Hz -> CQT bin %d, FFT bin %d\n", testFreqs[i], cqtBin, fftBin);
    }
    #endif
    
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
        
        // Check if kernel is valid
        if (!kernel->real || !kernel->imag || !kernel->indices || kernel->length == 0)
        {
            cqtData[bin] = 0.0f;
            continue;
        }
        
        float real = 0.0f;
        float imag = 0.0f;
        
        // Sparse matrix multiplication
        for (int k = 0; k < kernel->length; k++)
        {
            int idx = kernel->indices[k];
            // Ensure index is within bounds
            if (idx < 0 || idx > CQT_FFT_SIZE/2)
                continue;
                
            // Complex multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
            real += fftReal[idx] * kernel->real[k] - fftImag[idx] * kernel->imag[k];
            imag += fftReal[idx] * kernel->imag[k] + fftImag[idx] * kernel->real[k];
        }
        
        // Calculate magnitude with gain boost
        cqtData[bin] = sqrt(real * real + imag * imag) * 4.0f;  // Increased gain boost
        
        // Check for NaN or Inf
        if (!isfinite(cqtData[bin]))
            cqtData[bin] = 0.0f;
    }
}

// Process CQT from audio data
void CQT_ProcessAudio(void)
{
    if (!cqtFftCfg || !cqtEnabled) return;
    
    // Check if kernels are initialized
    bool kernelsValid = false;
    for (int i = 0; i < CQT_BINS; i++)
    {
        if (cqtKernels[i].real && cqtKernels[i].length > 0)
        {
            kernelsValid = true;
            break;
        }
    }
    
    if (!kernelsValid)
    {
        // Kernels not initialized, set all output to zero
        memset(cqtData, 0, sizeof(cqtData));
        memset(cqtSmoothingData, 0, sizeof(cqtSmoothingData));
        memset(cqtNormalizedData, 0, sizeof(cqtNormalizedData));
        return;
    }
    
    // Copy audio data from the shared buffer
    // TEMPORARY: sampleBuf now has FFT_SIZE * 2 = 2048 * 2 = 4096 samples
    memcpy(cqtAudioBuffer, sampleBuf, CQT_FFT_SIZE * sizeof(float));
    
    // Check if we have any audio data
    float audioSum = 0.0f;
    for (int i = 0; i < CQT_FFT_SIZE; i++)
    {
        audioSum += fabs(cqtAudioBuffer[i]);
    }
    
    if (audioSum < 0.0001f)
    {
        // No audio data, set output to zero
        memset(cqtData, 0, sizeof(cqtData));
        memset(cqtSmoothingData, 0, sizeof(cqtSmoothingData));
        memset(cqtNormalizedData, 0, sizeof(cqtNormalizedData));
        return;
    }
    
    // Perform 4096-point FFT
    kiss_fftr(cqtFftCfg, cqtAudioBuffer, cqtFftOutput);
    
    // Extract real and imaginary components for kernel application
    float fftReal[CQT_FFT_SIZE/2 + 1];
    float fftImag[CQT_FFT_SIZE/2 + 1];
    
    for (int i = 0; i <= CQT_FFT_SIZE/2; i++)
    {
        fftReal[i] = cqtFftOutput[i].r;
        fftImag[i] = cqtFftOutput[i].i;
    }
    
    // Apply CQT kernels
    CQT_ApplyKernels(fftReal, fftImag);
    
    #ifdef CQT_DEBUG
    // Reduced debug output - CQT is working correctly now
    static int debugCounter = 0;
    if (++debugCounter % 300 == 0)  // Print once every 5 seconds
    {
        // Find peak CQT bin
        int peakBin = 0;
        float peakVal = 0.0f;
        for (int i = 0; i < CQT_BINS; i++)
        {
            if (cqtData[i] > peakVal)
            {
                peakVal = cqtData[i];
                peakBin = i;
            }
        }
        
        if (peakVal > 1.0f)
        {
            float peakFreq = CQT_MIN_FREQ * pow(2.0f, peakBin / 12.0f);
            printf("CQT: Peak at bin %d (%.1f Hz) with magnitude %.1f\n", 
                   peakBin, peakFreq, peakVal);
        }
    }
    #endif
    
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
    
    // Initialize peak value if needed
    if (cqtPeakSmoothValue <= 0.0f)
        cqtPeakSmoothValue = 0.1f;
    
    // Smooth peak value
    if (currentPeak > cqtPeakSmoothValue)
        cqtPeakSmoothValue = currentPeak;
    else
        cqtPeakSmoothValue = cqtPeakSmoothValue * 0.99f + currentPeak * 0.01f;
    
    // Ensure peak value doesn't go too low
    if (cqtPeakSmoothValue < 0.0001f)
        cqtPeakSmoothValue = 0.0001f;
    
    // Normalize data
    float normalizer = 1.0f / cqtPeakSmoothValue;
    for (int i = 0; i < CQT_BINS; i++)
    {
        cqtNormalizedData[i] = cqtSmoothingData[i] * normalizer;
        if (cqtNormalizedData[i] > 1.0f) 
            cqtNormalizedData[i] = 1.0f;
        
        // Final NaN check
        if (!isfinite(cqtNormalizedData[i]))
            cqtNormalizedData[i] = 0.0f;
    }
}

// Process CQT from audio data (using existing FFT data from fftdata.h)
// This is kept for backward compatibility, but not used
void CQT_Process(const float* fftReal, const float* fftImag)
{
    if (!cqtFftCfg || !cqtEnabled) return;
    
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
    
    // Initialize peak value if needed
    if (cqtPeakSmoothValue <= 0.0f)
        cqtPeakSmoothValue = 0.1f;
    
    // Smooth peak value
    if (currentPeak > cqtPeakSmoothValue)
        cqtPeakSmoothValue = currentPeak;
    else
        cqtPeakSmoothValue = cqtPeakSmoothValue * 0.99f + currentPeak * 0.01f;
    
    // Ensure peak value doesn't go too low
    if (cqtPeakSmoothValue < 0.0001f)
        cqtPeakSmoothValue = 0.0001f;
    
    // Normalize data
    float normalizer = 1.0f / cqtPeakSmoothValue;
    for (int i = 0; i < CQT_BINS; i++)
    {
        cqtNormalizedData[i] = cqtSmoothingData[i] * normalizer;
        if (cqtNormalizedData[i] > 1.0f) 
            cqtNormalizedData[i] = 1.0f;
        
        // Final NaN check
        if (!isfinite(cqtNormalizedData[i]))
            cqtNormalizedData[i] = 0.0f;
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