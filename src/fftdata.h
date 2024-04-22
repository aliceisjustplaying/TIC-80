#pragma once
#define FFT_SIZE 1024
extern float fPeakMinValue;
extern float fPeakSmoothing;
extern float fPeakSmoothValue;
extern float fAmplification;
extern float fftData[FFT_SIZE];
extern float fftSmoothingData[FFT_SIZE];
extern float fftNormalizedData[FFT_SIZE];
extern float fftNormalizedMaxData[FFT_SIZE];

void FFT_DebugLog(const char* format, ...);