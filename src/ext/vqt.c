#include "vqt.h"
#include "vqt_kernel.h"
#include "../vqtdata.h"
#include "../fftdata.h"
#include "fft.h"
#include "kiss_fftr.h"
#include <math.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include <time.h>

#define VQT_DEBUG

// FFT configuration for VQT
static kiss_fftr_cfg vqtFftCfg = NULL;
static float* vqtAudioBuffer = NULL;
static kiss_fft_cpx* vqtFftOutput = NULL;

// Benchmark different FFT sizes
static void VQT_BenchmarkFFT(void)
{
    printf("\nVQT FFT Benchmark on this CPU:\n");
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

// Initialize VQT processing
bool VQT_Open(void)
{
    // Initialize data structures
    VQT_Init();
    
    // Allocate FFT buffers
    vqtAudioBuffer = (float*)malloc(VQT_FFT_SIZE * sizeof(float));
    vqtFftOutput = (kiss_fft_cpx*)malloc((VQT_FFT_SIZE/2 + 1) * sizeof(kiss_fft_cpx));
    
    if (!vqtAudioBuffer || !vqtFftOutput)
    {
        VQT_Close();
        return false;
    }
    
    // Create FFT configuration
    vqtFftCfg = kiss_fftr_alloc(VQT_FFT_SIZE, 0, NULL, NULL);
    if (!vqtFftCfg)
    {
        VQT_Close();
        return false;
    }
    
    // Configure VQT kernels
    VqtKernelConfig config = {
        .fftSize = VQT_FFT_SIZE,
        .numBins = VQT_BINS,
        .minFreq = VQT_MIN_FREQ,
        .maxFreq = VQT_MAX_FREQ,
        .sampleRate = 44100.0f,  // Standard TIC-80 sample rate
        .windowType = VQT_WINDOW_HAMMING,
        .sparsityThreshold = VQT_SPARSITY_THRESHOLD
    };
    
    // Generate kernels
    if (!VQT_GenerateKernels(vqtKernels, &config))
    {
        VQT_Close();
        return false;
    }
    
    // Run FFT benchmark on first initialization
    static bool benchmarkRun = false;
    if (!benchmarkRun)
    {
        VQT_BenchmarkFFT();
        benchmarkRun = true;
    }
    
    // Debug: Print variable Q values across frequency spectrum
    #ifdef VQT_DEBUG
    float centerFreqs[VQT_BINS];
    VQT_GenerateCenterFrequencies(centerFreqs, VQT_BINS, VQT_MIN_FREQ, VQT_MAX_FREQ);
    
    printf("\nVQT Variable-Q Implementation (8K FFT Optimized):\n");
    printf("================================================\n");
    
    // Show Q values for key frequency ranges
    float testFreqs[] = {20, 25, 30, 40, 50, 65, 80, 120, 160, 240, 320, 440, 640, 1000, 2000, 4000};
    printf("Frequency | Design Q | Window Length | Effective Q | Resolution\n");
    printf("----------|----------|---------------|-------------|------------\n");
    
    for (int i = 0; i < 16; i++)
    {
        float freq = testFreqs[i];
        // Find closest VQT bin
        int closestBin = 0;
        float minDiff = fabs(centerFreqs[0] - freq);
        for (int j = 1; j < VQT_BINS; j++)
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
        if (windowLength > VQT_FFT_SIZE) windowLength = VQT_FFT_SIZE;
        float effectiveQ = windowLength * freq / 44100.0f;
        float bandwidth = freq / effectiveQ;
        
        printf("%7.0f Hz | %7.1f | %13d | %11.1f | %7.1f Hz\n", 
               freq, designQ, windowLength, effectiveQ, bandwidth);
    }
    
    printf("\nFirst 10 VQT bins:\n");
    for (int i = 0; i < 10 && i < VQT_BINS; i++)
    {
        int expectedBin = (int)(centerFreqs[i] * VQT_FFT_SIZE / 44100.0f);
        printf("  Bin %d: %.2f Hz -> FFT bin %d\n", i, centerFreqs[i], expectedBin);
    }
    #endif
    
    return true;
}

// Apply VQT kernels to FFT output
void VQT_ApplyKernels(const float* fftReal, const float* fftImag)
{
    // Clear VQT output
    memset(vqtData, 0, sizeof(vqtData));
    
    // Apply each kernel to compute VQT bins
    for (int bin = 0; bin < VQT_BINS; bin++)
    {
        VqtKernel* kernel = &vqtKernels[bin];
        
        // Check if kernel is valid
        if (!kernel->real || !kernel->imag || !kernel->indices || kernel->length == 0)
        {
            vqtData[bin] = 0.0f;
            continue;
        }
        
        float real = 0.0f;
        float imag = 0.0f;
        
        // Sparse matrix multiplication
        for (int k = 0; k < kernel->length; k++)
        {
            int idx = kernel->indices[k];
            // Ensure index is within bounds
            if (idx < 0 || idx >= VQT_FFT_SIZE/2 + 1)
                continue;
                
            // Complex multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
            real += fftReal[idx] * kernel->real[k] - fftImag[idx] * kernel->imag[k];
            imag += fftReal[idx] * kernel->imag[k] + fftImag[idx] * kernel->real[k];
        }
        
        // Calculate magnitude with gain boost
        vqtData[bin] = sqrt(real * real + imag * imag) * 2.0f;  // Match FFT gain factor
        
        // Check for NaN or Inf
        if (!isfinite(vqtData[bin]))
            vqtData[bin] = 0.0f;
    }
}

// Process VQT from audio data
void VQT_ProcessAudio(void)
{
    if (!vqtFftCfg || !vqtEnabled) return;
    
    // Check if kernels are initialized
    bool kernelsValid = false;
    for (int i = 0; i < VQT_BINS; i++)
    {
        if (vqtKernels[i].real && vqtKernels[i].length > 0)
        {
            kernelsValid = true;
            break;
        }
    }
    
    if (!kernelsValid)
    {
        // Kernels not initialized, set all output to zero
        memset(vqtData, 0, sizeof(vqtData));
        memset(vqtSmoothingData, 0, sizeof(vqtSmoothingData));
        memset(vqtNormalizedData, 0, sizeof(vqtNormalizedData));
        return;
    }
    
    // Copy audio data from the shared buffer
    // sampleBuf is defined in fft.c as extern
    extern float sampleBuf[];
    memcpy(vqtAudioBuffer, sampleBuf, VQT_FFT_SIZE * sizeof(float));
    
    // Check if we have any audio data
    float audioSum = 0.0f;
    for (int i = 0; i < VQT_FFT_SIZE; i++)
    {
        audioSum += fabs(vqtAudioBuffer[i]);
    }
    
    if (audioSum < 0.0001f)
    {
        // No audio data, set output to zero
        memset(vqtData, 0, sizeof(vqtData));
        memset(vqtSmoothingData, 0, sizeof(vqtSmoothingData));
        memset(vqtNormalizedData, 0, sizeof(vqtNormalizedData));
        return;
    }
    
    // Profiling variables
    static double totalFftTime = 0.0;
    static double totalKernelTime = 0.0;
    static int profileCount = 0;
    
    // Perform 16384-point FFT with timing
    clock_t fftStart = clock();
    kiss_fftr(vqtFftCfg, vqtAudioBuffer, vqtFftOutput);
    clock_t fftEnd = clock();
    
    // Extract real and imaginary components for kernel application
    float fftReal[VQT_FFT_SIZE/2 + 1];
    float fftImag[VQT_FFT_SIZE/2 + 1];
    
    for (int i = 0; i <= VQT_FFT_SIZE/2; i++)
    {
        fftReal[i] = vqtFftOutput[i].r;
        fftImag[i] = vqtFftOutput[i].i;
    }
    
    // Apply VQT kernels with timing
    clock_t kernelStart = clock();
    VQT_ApplyKernels(fftReal, fftImag);
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
        printf("VQT Performance (16K FFT):\n");
        printf("  FFT avg: %.3fms\n", totalFftTime / profileCount);
        printf("  Kernels avg: %.3fms\n", totalKernelTime / profileCount);
        printf("  Total avg: %.3fms\n", (totalFftTime + totalKernelTime) / profileCount);
        printf("  Samples: %d\n\n", profileCount);
    }
    
    #ifdef VQT_DEBUG
    // Reduced debug output - VQT is working correctly now
    static int debugCounter = 0;
    if (++debugCounter % 300 == 0)  // Print once every 5 seconds
    {
        // Find peak VQT bin
        int peakBin = 0;
        float peakVal = 0.0f;
        for (int i = 0; i < VQT_BINS; i++)
        {
            if (vqtData[i] > peakVal)
            {
                peakVal = vqtData[i];
                peakBin = i;
            }
        }
        
        if (peakVal > 1.0f)
        {
            float peakFreq = VQT_MIN_FREQ * pow(2.0f, peakBin / 12.0f);
            printf("VQT: Peak at bin %d (%.1f Hz) with magnitude %.1f\n", 
                   peakBin, peakFreq, peakVal);
        }
    }
    #endif
    
    // Apply smoothing to raw data
    for (int i = 0; i < VQT_BINS; i++)
    {
        vqtSmoothingData[i] = vqtSmoothingData[i] * VQT_SMOOTHING_FACTOR + 
                              vqtData[i] * (1.0f - VQT_SMOOTHING_FACTOR);
    }
    
    // Find peak for normalization
    float currentPeak = 0.0f;
    for (int i = 0; i < VQT_BINS; i++)
    {
        if (vqtSmoothingData[i] > currentPeak)
            currentPeak = vqtSmoothingData[i];
    }
    
    // Initialize peak value if needed
    if (vqtPeakSmoothValue <= 0.0f)
        vqtPeakSmoothValue = 0.1f;
    
    // Smooth peak value
    if (currentPeak > vqtPeakSmoothValue)
        vqtPeakSmoothValue = currentPeak;
    else
        vqtPeakSmoothValue = vqtPeakSmoothValue * 0.99f + currentPeak * 0.01f;
    
    // Ensure peak value doesn't go too low
    if (vqtPeakSmoothValue < 0.0001f)
        vqtPeakSmoothValue = 0.0001f;
    
    // Normalize data
    float normalizer = 1.0f / vqtPeakSmoothValue;
    for (int i = 0; i < VQT_BINS; i++)
    {
        vqtNormalizedData[i] = vqtSmoothingData[i] * normalizer;
        if (vqtNormalizedData[i] > 1.0f) 
            vqtNormalizedData[i] = 1.0f;
        
        // Final NaN check
        if (!isfinite(vqtNormalizedData[i]))
            vqtNormalizedData[i] = 0.0f;
    }
}


// Close VQT processing and free resources
void VQT_Close(void)
{
    if (vqtFftCfg)
    {
        free(vqtFftCfg);
        vqtFftCfg = NULL;
    }
    
    if (vqtAudioBuffer)
    {
        free(vqtAudioBuffer);
        vqtAudioBuffer = NULL;
    }
    
    if (vqtFftOutput)
    {
        free(vqtFftOutput);
        vqtFftOutput = NULL;
    }
    
    // Clean up kernels
    VQT_Cleanup();
}