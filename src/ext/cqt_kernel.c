#include "cqt_kernel.h"
#include "../cqtdata.h"
#include <math.h>
#include <stdlib.h>
#include <string.h>
#include "../ext/kiss_fftr.h"

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

// Generate logarithmically spaced center frequencies for musical notes
void CQT_GenerateCenterFrequencies(float* frequencies, int numBins, float minFreq, float maxFreq)
{
    // For musical CQT with 12 bins per octave, we want equal temperament spacing
    // Each semitone is a factor of 2^(1/12) ≈ 1.0594631
    const float semitone = pow(2.0f, 1.0f / 12.0f);
    
    for (int i = 0; i < numBins; i++)
    {
        // Calculate frequency for each bin based on semitone spacing
        frequencies[i] = minFreq * pow(semitone, i);
    }
    
    // Verify we don't exceed maxFreq
    if (frequencies[numBins - 1] > maxFreq)
    {
        // Scale down if necessary
        float scale = maxFreq / frequencies[numBins - 1];
        for (int i = 0; i < numBins; i++)
        {
            frequencies[i] *= scale;
        }
    }
}

// Calculate Q factor for constant-Q transform
float CQT_CalculateQ(int binsPerOctave)
{
    // Q = 1 / (2^(1/binsPerOctave) - 1)
    // For 12 bins/octave, Q ≈ 17.0
    return 1.0f / (pow(2.0, 1.0 / binsPerOctave) - 1.0f);
}

// Generate Hamming window
static void generateHammingWindow(float* window, int length)
{
    const float a0 = 0.54f;
    const float a1 = 0.46f;
    
    for (int i = 0; i < length; i++)
    {
        window[i] = a0 - a1 * cos(2.0 * M_PI * i / (length - 1));
    }
}

// Generate Gaussian window
static void generateGaussianWindow(float* window, int length)
{
    const float sigma = 0.5f; // Standard deviation factor
    float center = (length - 1) / 2.0f;
    float normFactor = sigma * length / 2.0f;
    
    for (int i = 0; i < length; i++)
    {
        float x = (i - center) / normFactor;
        window[i] = exp(-0.5f * x * x);
    }
}

// Generate a single CQT kernel
static bool generateSingleKernel(
    CqtKernel* kernel,
    kiss_fftr_cfg fftCfg,
    int fftSize,
    float centerFreq,
    float minFreq,
    float sampleRate,
    CqtWindowType windowType,
    float sparsityThreshold)
{
    // Calculate window length based on Q factor
    // windowLength = Q * sampleRate / centerFreq
    // This gives us the proper frequency resolution
    float Q = CQT_CalculateQ(CQT_BINS_PER_OCTAVE);
    int windowLength = (int)(Q * sampleRate / centerFreq);
    
    // Ensure window length is reasonable
    if (windowLength < 32) windowLength = 32;  // Minimum window size
    if (windowLength > fftSize) windowLength = fftSize;
    
    // Allocate temporary arrays
    float* timeKernel = (float*)calloc(fftSize, sizeof(float));
    kiss_fft_cpx* freqKernel = (kiss_fft_cpx*)malloc((fftSize/2 + 1) * sizeof(kiss_fft_cpx));
    
    if (!timeKernel || !freqKernel)
    {
        free(timeKernel);
        free(freqKernel);
        return false;
    }
    
    // Generate window in the center of the kernel
    int windowStart = (fftSize - windowLength) / 2;
    
    // First generate the window
    float* tempWindow = (float*)malloc(windowLength * sizeof(float));
    if (!tempWindow)
    {
        free(timeKernel);
        free(freqKernel);
        return false;
    }
    
    switch (windowType)
    {
        case CQT_WINDOW_HAMMING:
            generateHammingWindow(tempWindow, windowLength);
            break;
        case CQT_WINDOW_GAUSSIAN:
            generateGaussianWindow(tempWindow, windowLength);
            break;
    }
    
    // Apply window and modulate with complex exponential
    // Following ESP32 reference: modulate based on full FFT size, not window size
    for (int i = 0; i < windowLength; i++)
    {
        int idx = windowStart + i;
        // Key insight: use idx (position in full FFT buffer) not i (position in window)
        // and center around N/2 where N is the full FFT size
        float phase = 2.0f * M_PI * (centerFreq / sampleRate) * (idx - fftSize/2);
        timeKernel[idx] = tempWindow[i] * cos(phase);
    }
    
    // Normalize by window length before FFT (like ESP32)
    for (int i = 0; i < fftSize; i++)
    {
        timeKernel[i] /= windowLength;
    }
    
    free(tempWindow);
    
    // Perform FFT
    kiss_fftr(fftCfg, timeKernel, freqKernel);
    
    // Count non-zero elements (above sparsity threshold)
    int nonZeroCount = 0;
    for (int i = 0; i <= fftSize/2; i++)
    {
        float magnitude = sqrt(freqKernel[i].r * freqKernel[i].r + 
                              freqKernel[i].i * freqKernel[i].i);
        if (magnitude > sparsityThreshold)
        {
            nonZeroCount++;
        }
    }
    
    // Allocate sparse storage
    kernel->real = (float*)malloc(nonZeroCount * sizeof(float));
    kernel->imag = (float*)malloc(nonZeroCount * sizeof(float));
    kernel->indices = (int*)malloc(nonZeroCount * sizeof(int));
    kernel->length = nonZeroCount;
    
    if (!kernel->real || !kernel->imag || !kernel->indices)
    {
        free(kernel->real);
        free(kernel->imag);
        free(kernel->indices);
        free(timeKernel);
        free(freqKernel);
        return false;
    }
    
    // Store non-zero elements
    int sparseIndex = 0;
    int minIdx = fftSize, maxIdx = 0;  // Track range for debug
    for (int i = 0; i <= fftSize/2; i++)
    {
        float magnitude = sqrt(freqKernel[i].r * freqKernel[i].r + 
                              freqKernel[i].i * freqKernel[i].i);
        if (magnitude > sparsityThreshold)
        {
            kernel->real[sparseIndex] = freqKernel[i].r;
            kernel->imag[sparseIndex] = freqKernel[i].i;
            kernel->indices[sparseIndex] = i;
            if (i < minIdx) minIdx = i;
            if (i > maxIdx) maxIdx = i;
            sparseIndex++;
        }
    }
    
    #ifdef CQT_DEBUG
    // Debug output for specific frequencies
    if (fabs(centerFreq - 110.0f) < 1.0f || fabs(centerFreq - 440.0f) < 1.0f)
    {
        printf("Kernel for %.1f Hz: FFT bins %d-%d (count: %d)\n", 
               centerFreq, minIdx, maxIdx, nonZeroCount);
    }
    #endif
    
    free(timeKernel);
    free(freqKernel);
    return true;
}

// Generate CQT kernels for all bins
bool CQT_GenerateKernels(CqtKernel* kernels, const CqtKernelConfig* config)
{
    // Generate center frequencies
    float* centerFreqs = (float*)malloc(config->numBins * sizeof(float));
    if (!centerFreqs) return false;
    
    CQT_GenerateCenterFrequencies(centerFreqs, config->numBins, 
                                  config->minFreq, config->maxFreq);
    
    // Create FFT configuration
    kiss_fftr_cfg fftCfg = kiss_fftr_alloc(config->fftSize, 0, NULL, NULL);
    if (!fftCfg)
    {
        free(centerFreqs);
        return false;
    }
    
    // Generate kernel for each CQT bin
    bool success = true;
    for (int i = 0; i < config->numBins; i++)
    {
        if (!generateSingleKernel(&kernels[i], fftCfg, config->fftSize,
                                 centerFreqs[i], config->minFreq, 
                                 config->sampleRate, config->windowType,
                                 config->sparsityThreshold))
        {
            // Clean up on failure
            for (int j = 0; j < i; j++)
            {
                free(kernels[j].real);
                free(kernels[j].imag);
                free(kernels[j].indices);
            }
            success = false;
            break;
        }
    }
    
    free(fftCfg);
    free(centerFreqs);
    return success;
}