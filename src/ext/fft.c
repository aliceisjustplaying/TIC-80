// #define MA_DEBUG_OUTPUT
#define MINIAUDIO_IMPLEMENTATION
#include <stdio.h>
#include <memory.h>
#include "miniaudio.h"
#include "kiss_fft.h"
#include "kiss_fftr.h"
#include "api.h"

#include "../fftdata.h"
#include "fft.h"

//////////////////////////////////////////////////////////////////////////

kiss_fftr_cfg fftcfg;
ma_context context;
ma_device captureDevice;
float sampleBuf[FFT_SIZE * 2];

void miniaudioLogCallback(void *userData, ma_uint32 level, const char *message)
{
    FFT_DebugLog(FFT_LOG_TRACE, "miniaudioLogCallback got called\n");
    // TODO: I don't know why we need this or if we even need this
    (void)userData;
    switch (level) {
    case MA_LOG_LEVEL_DEBUG:
        printf( "[MA DEBUG]: %s", message );
        break;
    case MA_LOG_LEVEL_INFO:
        printf( "[MA INFO]: %s", message );
        break;
    case MA_LOG_LEVEL_WARNING:
        printf( "[MA WARNING]: %s", message );
        break;
    case MA_LOG_LEVEL_ERROR:
        printf( "[MA ERROR]: %s", message );
        break;
    }

    return;
}

void OnReceiveFrames(ma_device* pDevice, void* pOutput, const void* pInput, ma_uint32 frameCount)
{
    frameCount = frameCount < FFT_SIZE * 2 ? frameCount : FFT_SIZE * 2;

  // Just rotate the buffer; copy existing, append new
    const float* samples = (const float*)pInput;
    float* p = sampleBuf;
    for (int i = 0; i < FFT_SIZE * 2 - frameCount; i++)
    {
        *(p++) = sampleBuf[i + frameCount];
    }
    for (int i = 0; i < frameCount; i++)
    {
        *(p++) = (samples[i * 2] + samples[i * 2 + 1]) / 2.0f * fAmplification;
    }
}

void FFT_EnumerateDevices()
{
  ma_context_config context_config = ma_context_config_init();
  ma_log log;
  ma_log_init(NULL, &log);
  ma_log_register_callback(&log, ma_log_callback_init(miniaudioLogCallback, NULL));

  context_config.pLog = &log;

  ma_result result = ma_context_init(NULL, 0, &context_config, &context);
  if (result != MA_SUCCESS)
  {
    FFT_DebugLog(FFT_LOG_ERROR, "[FFT] Failed to initialize context: %s", ma_result_description(result));
    return;
  }

  FFT_DebugLog(FFT_LOG_INFO, "MAL context initialized, backend is '%s'\n", ma_get_backend_name( context.backend ) );

  ma_device_info* pPlaybackDeviceInfos;
  ma_uint32 playbackDeviceCount;
  ma_device_info* pCaptureDeviceInfos;
  ma_uint32 captureDeviceCount;
  result = ma_context_get_devices(&context, &pPlaybackDeviceInfos, &playbackDeviceCount, &pCaptureDeviceInfos, &captureDeviceCount);
  if (result != MA_SUCCESS) {
      FFT_DebugLog(FFT_LOG_ERROR, "Failed to retrieve device information.\n");
      FFT_DebugLog(FFT_LOG_ERROR, "Error: %s\n", ma_result_description(result));
      return;
  }

  FFT_DebugLog(FFT_LOG_INFO, "Playback Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < playbackDeviceCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pPlaybackDeviceInfos[iDevice].name);
  }
  
  printf("\n");

  FFT_DebugLog(FFT_LOG_INFO, "Capture Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < captureDeviceCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pCaptureDeviceInfos[iDevice].name);
  }

  printf("\n");

  return;
}

bool FFT_Open(bool CapturePlaybackDevices, const char* CaptureDeviceSearchString)
{
  memset( sampleBuf, 0, sizeof( float ) * FFT_SIZE * 2 );

  fftcfg = kiss_fftr_alloc( FFT_SIZE * 2, false, NULL, NULL );

  ma_context_config context_config = ma_context_config_init();
  ma_log log;
  ma_log_init(NULL, &log);
  ma_log_register_callback(&log, ma_log_callback_init(miniaudioLogCallback, NULL));

  context_config.pLog = &log;

  ma_result result = ma_context_init( NULL, 0, &context_config, &context );
  if ( result != MA_SUCCESS )
  {
    FFT_DebugLog(FFT_LOG_ERROR, "Failed to initialize context: %d", result );
    return false;
  }

  FFT_DebugLog(FFT_LOG_INFO, "MAL context initialized, backend is '%s'\n", ma_get_backend_name( context.backend ) );

  ma_device_id* TargetDevice = NULL;

  ma_device_info* pPlaybackDeviceInfos;
  ma_uint32 playbackDeviceCount;
  ma_device_info* pCaptureDeviceInfos;
  ma_uint32 captureDeviceCount;
  result = ma_context_get_devices(&context, &pPlaybackDeviceInfos, &playbackDeviceCount, &pCaptureDeviceInfos, &captureDeviceCount);
  if (result != MA_SUCCESS) {
      FFT_DebugLog(FFT_LOG_ERROR, "Failed to retrieve device information.\n");
      return false;
  }

  FFT_DebugLog(FFT_LOG_INFO, "Playback Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < playbackDeviceCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pPlaybackDeviceInfos[iDevice].name);
  }
  
  printf("\n");

  FFT_DebugLog(FFT_LOG_INFO, "Capture Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < captureDeviceCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pCaptureDeviceInfos[iDevice].name);
  }

  printf("\n");

  bool useLoopback = (ma_is_loopback_supported(context.backend) && CapturePlaybackDevices);
  FFT_DebugLog(FFT_LOG_INFO, "Loopback support: %s, Use loopback: %s\n", ma_is_loopback_supported(context.backend) ? "Yes" : "No", useLoopback ? "Yes" : "No");
  
  if(CaptureDeviceSearchString && strlen(CaptureDeviceSearchString) > 0) {
    if(useLoopback) {
      for (ma_uint32 iDevice = 0; iDevice < playbackDeviceCount; ++iDevice) {
        char* DeviceName = pPlaybackDeviceInfos[iDevice].name;
        if(strstr(DeviceName, CaptureDeviceSearchString) != NULL) {
          TargetDevice = &pPlaybackDeviceInfos[iDevice].id;
          break;
        }
      }
    } else {
      for (ma_uint32 iDevice = 0; iDevice < captureDeviceCount; ++iDevice) {
        char* DeviceName = pCaptureDeviceInfos[iDevice].name;
        if(strstr(DeviceName, CaptureDeviceSearchString) != NULL) {
          TargetDevice = &pCaptureDeviceInfos[iDevice].id;
          break;
        }
      }
    }
  }

  ma_device_config config = ma_device_config_init( useLoopback ? ma_device_type_loopback : ma_device_type_capture );
  config.capture.pDeviceID = TargetDevice;
  config.capture.format = ma_format_f32;
  config.capture.channels = 2;
  config.sampleRate = 11025;
  config.dataCallback = OnReceiveFrames;
  config.pUserData = NULL;

  result = ma_device_init( &context, &config, &captureDevice );
  if ( result != MA_SUCCESS )
  {
    ma_context_uninit( &context );
    FFT_DebugLog(FFT_LOG_ERROR, "Failed to initialize capture device: %d\n", result );
    return false;
  }

  result = ma_device_start( &captureDevice );
  if ( result != MA_SUCCESS )
  {
    ma_device_uninit( &captureDevice );
    ma_context_uninit( &context );
    FFT_DebugLog(FFT_LOG_ERROR, "Failed to start capture device: %d\n", result );
    return false;
  }

  FFT_DebugLog(FFT_LOG_INFO, "Capturing %s\n", captureDevice.capture.name );
  
  fftEnabled = true;
  return true;
}

void FFT_Close()
{
  ma_device_stop(&captureDevice);
  ma_device_uninit(&captureDevice);
  ma_context_uninit(&context);
  kiss_fft_free(fftcfg);
  fftEnabled = false;
}

//////////////////////////////////////////////////////////////////////////

bool FFT_GetFFT(float* _samples)
{
  kiss_fft_cpx out[FFT_SIZE + 1];
  kiss_fftr(fftcfg, sampleBuf, out);

  bool bPeakNormalization = true;
  if (bPeakNormalization) {
    float peakValue = fPeakMinValue;
    for (int i = 0; i < FFT_SIZE; i++)
    {
      float val = 2.0f * sqrtf(out[i].r * out[i].r + out[i].i * out[i].i);
      if (val > peakValue) peakValue = val;
      _samples[i] = val * fAmplification;
    }
    if (peakValue > fPeakSmoothValue) {
      fPeakSmoothValue = peakValue;
    }
    if (peakValue < fPeakSmoothValue) {
      fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
    }
    fAmplification = 1.0f / fPeakSmoothValue;
  } else {
    for (int i = 0; i < FFT_SIZE; i++)
    {
      static const float scaling = 1.0f / (float)FFT_SIZE;
      _samples[i] = 2.0f * sqrtf(out[i].r * out[i].r + out[i].i * out[i].i) * scaling * fAmplification;
    }
  }

  float fFFTSmoothingFactor = 0.6f;
  bool bSmoothing = true;
  if (bSmoothing)
  {
    for ( int i = 0; i < FFT_SIZE; i++ )
    {
      fftSmoothingData[i] = fftSmoothingData[i] * fFFTSmoothingFactor + (1 - fFFTSmoothingFactor) * _samples[i];
    }
  }

  bool bNormalization = true;
  if (bNormalization)
  {
    for ( int i = 0; i < FFT_SIZE; i++ )
    {
        float v = fftSmoothingData[i];
        fftNormalizedMaxData[i] = MAX(fftNormalizedMaxData[i], v);
        float min = 0.001;
        fftNormalizedData[i] = v / MAX(min, fftNormalizedMaxData[i]);
        fftNormalizedMaxData[i] = fftNormalizedMaxData[i] * 0.9999;
    }
  }

  return true;
}

//////////////////////////////////////////////////////////////////////////

double tic_api_fft(tic_mem* memory, s32 freq)
{
  if (!fftEnabled)
  {
    FFT_DebugLog(FFT_LOG_TRACE, "tic_api_fft: fft not enabled\n");
    return 0.0;
  }

  if (freq < 0 || freq >= FFT_SIZE)
  {
    FFT_DebugLog(FFT_LOG_TRACE, "tic_api_fft: freq out of bounds at %d\n", freq);
    return 0.0;
  }
  
  return fftData[freq];
}

double tic_api_ffts(tic_mem* memory, s32 freq)
{
  if (!fftEnabled)
  {
    FFT_DebugLog(FFT_LOG_TRACE, "tic_api_ffts: fft not enabled\n");
    return 0.0;
  }

  if (freq < 0 || freq >= FFT_SIZE)
  {
    FFT_DebugLog(FFT_LOG_TRACE, "tic_api_ffts: freq out of bounds at %d\n", freq);
    return 0.0;
  }
  
  return fftSmoothingData[freq];
}
