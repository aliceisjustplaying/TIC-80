#pragma once
#include <stdbool.h>


// typedef int                 BOOL;
  //////////////////////////////////////////////////////////////////////////

  typedef struct FFT_Settings
  {
    bool bUseRecordingDevice;
    void* pDeviceID;
  } FFT_Settings;

  typedef void (*FFT_ENUMERATE_FUNC)(const bool bIsCaptureDevice, const char* szDeviceName, void* pDeviceID, void* pUserContext);

  void FFT_EnumerateDevices(FFT_ENUMERATE_FUNC pEnumerationFunction, void* pUserContext);

  bool FFT_Create();
  bool FFT_Destroy();
  bool FFT_Open(FFT_Settings* pSettings);
  bool FFT_GetFFT(float* _samples);
  void FFT_Close();

  //////////////////////////////////////////////////////////////////////////
