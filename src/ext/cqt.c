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
#include <time.h>

#define CQT_DEBUG

// FFT configuration for CQT
static kiss_fftr_cfg cqtFftCfg = NULL;
static float* cqtAudioBuffer = NULL;
static kiss_fft_cpx* cqtFftOutput = NULL;

// Benchmark different FFT sizes
static void CQT_BenchmarkFFT(void)
{
    printf("\nCQT FFT Benchmark on this CPU:\n");
    printf("================================\n");
    
    int sizes[] = {4096, 6144, 8192, 12288, 16384, 24576, 32768};
    int numSizes = 7;
    
    for (int s = 0; s < numSizes; s++)
    {
        int fftSize = sizes[s];
        
        // Allocate buffers
        float* testInput = (float*)calloc(fftSize, sizeof(float));
        kiss_fft_cpx* testOutput = (kiss_fft_cpx*)malloc((fftSize/2 + 1) * sizeof(kiss_fft_cpx));
        kiss_fftr_cfg testCfg = kiss_fftr_alloc(fftSize, 0, NULL, NULL);
        
        if (!testInput || !testOutput || !testCfg)
        {
            printf("Failed to allocate for size %d\n", fftSize);
            continue;
        }
        
        // Fill with test signal
        for (int i = 0; i < fftSize; i++)
        {
            testInput[i] = sin(2.0 * M_PI * 440.0 * i / 44100.0);
        }
        
        // Warm up
        for (int i = 0; i < 10; i++)
        {
            kiss_fftr(testCfg, testInput, testOutput);
        }
        
        // Time 100 iterations
        clock_t start = clock();
        for (int i = 0; i < 100; i++)
        {
            kiss_fftr(testCfg, testInput, testOutput);
        }
        clock_t end = clock();
        
        double avgTime = (double)(end - start) / CLOCKS_PER_SEC * 1000.0 / 100.0;
        printf("%5d-point FFT: %.3f ms\n", fftSize, avgTime);
        
        // Cleanup
        free(testInput);
        free(testOutput);
        free(testCfg);
    }
    
    printf("================================\n\n");
}

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
    
    // Run FFT benchmark on first initialization
    static bool benchmarkRun = false;
    if (!benchmarkRun)
    {
        CQT_BenchmarkFFT();
        benchmarkRun = true;
    }
    
    // Debug: Print variable Q values across frequency spectrum
    #ifdef CQT_DEBUG
    float centerFreqs[CQT_BINS];
    CQT_GenerateCenterFrequencies(centerFreqs, CQT_BINS, CQT_MIN_FREQ, CQT_MAX_FREQ);
    
    printf("\nCQT Variable-Q Implementation (8K FFT Optimized):\n");
    printf("================================================\n");
    
    // Show Q values for key frequency ranges
    float testFreqs[] = {20, 25, 30, 40, 50, 65, 80, 120, 160, 240, 320, 440, 640, 1000, 2000, 4000};
    printf("Frequency | Design Q | Window Length | Effective Q | Resolution\n");
    printf("----------|----------|---------------|-------------|------------\n");
    
    for (int i = 0; i < 16; i++)
    {
        float freq = testFreqs[i];
        // Find closest CQT bin
        int closestBin = 0;
        float minDiff = fabs(centerFreqs[0] - freq);
        for (int j = 1; j < CQT_BINS; j++)
        {
            float diff = fabs(centerFreqs[j] - freq);
            if (diff < minDiff)
            {
                minDiff = diff;
                closestBin = j;
            }
        }
        
        // Calculate Q values using 8K-optimized function
        float designQ;
        if (freq < 25.0f) designQ = 7.4f;
        else if (freq < 30.0f) designQ = 9.2f;
        else if (freq < 40.0f) designQ = 11.5f;
        else if (freq < 50.0f) designQ = 14.5f;
        else if (freq < 65.0f) designQ = 16.0f;
        else if (freq < 80.0f) designQ = 17.0f;
        else if (freq < 160.0f) designQ = 17.0f;
        else if (freq < 320.0f) designQ = 15.0f;
        else if (freq < 640.0f) designQ = 13.0f;
        else designQ = 11.0f;
        
        int windowLength = (int)(designQ * 44100.0f / freq);
        if (windowLength > CQT_FFT_SIZE) windowLength = CQT_FFT_SIZE;
        float effectiveQ = windowLength * freq / 44100.0f;
        float bandwidth = freq / effectiveQ;
        
        printf("%7.0f Hz | %7.1f | %13d | %11.1f | %7.1f Hz\n", 
               freq, designQ, windowLength, effectiveQ, bandwidth);
    }
    
    printf("\nFirst 10 CQT bins:\n");
    for (int i = 0; i < 10 && i < CQT_BINS; i++)
    {
        int expectedBin = (int)(centerFreqs[i] * CQT_FFT_SIZE / 44100.0f);
        printf("  Bin %d: %.2f Hz -> FFT bin %d\n", i, centerFreqs[i], expectedBin);
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
            if (idx < 0 || idx >= CQT_FFT_SIZE/2 + 1)
                continue;
                
            // Complex multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
            real += fftReal[idx] * kernel->real[k] - fftImag[idx] * kernel->imag[k];
            imag += fftReal[idx] * kernel->imag[k] + fftImag[idx] * kernel->real[k];
        }
        
        // Calculate magnitude with gain boost
        cqtData[bin] = sqrt(real * real + imag * imag) * 2.0f;  // Match FFT gain factor
        
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
    // sampleBuf is defined in fft.c as extern
    extern float sampleBuf[];
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
    
    // Profiling variables
    static double totalFftTime = 0.0;
    static double totalKernelTime = 0.0;
    static int profileCount = 0;
    
    // Perform 16384-point FFT with timing
    clock_t fftStart = clock();
    kiss_fftr(cqtFftCfg, cqtAudioBuffer, cqtFftOutput);
    clock_t fftEnd = clock();
    
    // Extract real and imaginary components for kernel application
    float fftReal[CQT_FFT_SIZE/2 + 1];
    float fftImag[CQT_FFT_SIZE/2 + 1];
    
    for (int i = 0; i <= CQT_FFT_SIZE/2; i++)
    {
        fftReal[i] = cqtFftOutput[i].r;
        fftImag[i] = cqtFftOutput[i].i;
    }
    
    // Apply CQT kernels with timing
    clock_t kernelStart = clock();
    CQT_ApplyKernels(fftReal, fftImag);
    clock_t kernelEnd = clock();
    
    // Calculate times in milliseconds
    double fftTime = (double)(fftEnd - fftStart) / CLOCKS_PER_SEC * 1000.0;
    double kernelTime = (double)(kernelEnd - kernelStart) / CLOCKS_PER_SEC * 1000.0;
    
    totalFftTime += fftTime;
    totalKernelTime += kernelTime;
    profileCount++;
    
    // Print profiling info every 60 frames (~1 second)
    if (profileCount % 60 == 0)
    {
        printf("CQT Performance (16K FFT):\n");
        printf("  FFT avg: %.3fms\n", totalFftTime / profileCount);
        printf("  Kernels avg: %.3fms\n", totalKernelTime / profileCount);
        printf("  Total avg: %.3fms\n", (totalFftTime + totalKernelTime) / profileCount);
        printf("  Samples: %d\n\n", profileCount);
    }
    
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
    
    // Apply spectral whitening if enabled
    #if CQT_SPECTRAL_WHITENING_ENABLED
    for (int i = 0; i < CQT_BINS; i++)
    {
        // Update running average for this bin
        cqtBinAverages[i] = cqtBinAverages[i] * CQT_WHITENING_DECAY + 
                            cqtData[i] * (1.0f - CQT_WHITENING_DECAY);
        
        // Ensure average doesn't go below floor
        if (cqtBinAverages[i] < CQT_WHITENING_FLOOR)
            cqtBinAverages[i] = CQT_WHITENING_FLOOR;
        
        // Apply whitening by dividing by the average
        cqtWhitenedData[i] = cqtData[i] / cqtBinAverages[i];
        
        // Check for NaN or Inf
        if (!isfinite(cqtWhitenedData[i]))
            cqtWhitenedData[i] = 0.0f;
    }
    
    // Apply smoothing to whitened data
    for (int i = 0; i < CQT_BINS; i++)
    {
        cqtSmoothingData[i] = cqtSmoothingData[i] * CQT_SMOOTHING_FACTOR + 
                              cqtWhitenedData[i] * (1.0f - CQT_SMOOTHING_FACTOR);
    }
    #else
    // Apply smoothing to raw data (spectral whitening disabled)
    for (int i = 0; i < CQT_BINS; i++)
    {
        cqtSmoothingData[i] = cqtSmoothingData[i] * CQT_SMOOTHING_FACTOR + 
                              cqtData[i] * (1.0f - CQT_SMOOTHING_FACTOR);
    }
    #endif
    
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