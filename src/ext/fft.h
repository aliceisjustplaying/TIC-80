#pragma once
#include <stdbool.h>

//////////////////////////////////////////////////////////////////////////

typedef struct FFT_Settings
{
  bool bUseRecordingDevice;
  void* pDeviceID;
} FFT_Settings;

// typedef void (*FFT_ENUMERATE_FUNC)(const bool bIsCaptureDevice, const char* szDeviceName, void* pDeviceID, void* pUserContext);

bool FFT_Open(bool CapturePlaybackDevices, const char* CaptureDeviceSearchString);
// void FFT_EnumerateDevices(FFT_ENUMERATE_FUNC pEnumerationFunction, void* pUserContext);
void FFT_EnumerateDevices();
bool FFT_GetFFT(float* _samples);
void FFT_Close();

//////////////////////////////////////////////////////////////////////////
