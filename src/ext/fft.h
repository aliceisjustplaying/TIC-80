#pragma once
#define FFT_SIZE 2048
#include <stdbool.h>

  //////////////////////////////////////////////////////////////////////////

  typedef struct FFT_Settings
  {
    bool bUseRecordingDevice;
    void* pDeviceID;
  } FFT_Settings;

  typedef void (*FFT_ENUMERATE_FUNC)(const bool bIsCaptureDevice, const char* szDeviceName, void* pDeviceID, void* pUserContext);

  extern float fAmplification;

  void FFT_EnumerateDevices(FFT_ENUMERATE_FUNC pEnumerationFunction, void* pUserContext);

  bool FFT_Create();
  bool FFT_Destroy();
  bool FFT_Open(FFT_Settings* pSettings);
  void FFT_Close();

  //////////////////////////////////////////////////////////////////////////
