#include "fftdata.h"
#include <stdarg.h> // For va_list
#include <stdio.h> // for vprintf

#define FFT_DEBUG

void FFT_DebugLog(const char* format, ...)
{
#ifdef FFT_DEBUG
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
#endif
}
float fPeakMinValue = 0.01f;
float fPeakSmoothing = 0.995f;
float fPeakSmoothValue = 0.0f;
float fAmplification = 1.0f;
float fftData[FFT_SIZE] = {0};
float fftSmoothingData[FFT_SIZE] = {0};
float fftNormalizedData[FFT_SIZE] = {0};
float fftNormalizedMaxData[FFT_SIZE] = {0};