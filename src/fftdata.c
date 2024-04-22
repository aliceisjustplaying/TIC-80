#include "fftdata.h"
float fPeakMinValue = 0.01f;
float fPeakSmoothing = 0.995f;
float fPeakSmoothValue = 0.0f;
float fAmplification = 1.0f;
float fftData[FFT_SIZE] = {0};
float fftSmoothingData[FFT_SIZE] = {0};
float fftNormalizedData[FFT_SIZE] = {0};
float fftNormalizedMaxData[FFT_SIZE] = {0};
