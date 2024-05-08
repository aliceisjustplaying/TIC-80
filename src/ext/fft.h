#pragma once
#include <stdbool.h>
#define FFT_SIZE 2048

//////////////////////////////////////////////////////////////////////////

typedef enum {
    FFT_LOG_OFF = 0,
    FFT_LOG_FATAL,
    FFT_LOG_ERROR,
    FFT_LOG_WARNING,
    FFT_LOG_INFO,
    FFT_LOG_DEBUG,
    FFT_LOG_TRACE
} FFT_LogLevel;

extern FFT_LogLevel g_currentLogLevel;

void FFT_DebugLog(FFT_LogLevel level, const char *format, ...);

//////////////////////////////////////////////////////////////////////////

bool FFT_Open(bool CapturePlaybackDevices, const char* CaptureDeviceSearchString);
void FFT_EnumerateDevices();
// double tic_api_fft(tic_mem* memory, s32 freq);
void FFT_Close();

//////////////////////////////////////////////////////////////////////////

