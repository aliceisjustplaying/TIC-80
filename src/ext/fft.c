#define MA_DEBUG_OUTPUT
#define MINIAUDIO_IMPLEMENTATION
#include "fft.h"
#include "miniaudio.h"

#include "kiss_fft.h"
#include "kiss_fftr.h"
#include <stdio.h>
#include <memory.h>
#include "api.h"
#include "fftdata.h"

//////////////////////////////////////////////////////////////////////////

kiss_fftr_cfg fftcfg;
ma_context context;
ma_device captureDevice;
float sampleBuf[FFT_SIZE * 2];
bool bCreated = false;
kiss_fft_cpx fftBuf[FFT_SIZE + 1];

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

  // kiss_fftr(fftcfg, sampleBuf, fftBuf);
}

// void FFT_EnumerateDevices(FFT_ENUMERATE_FUNC pEnumerationFunction, void* pUserContext)
// {
//   if (!bCreated)
//   {
//     return;
//   }

//   ma_device_info* pPlaybackDevices = NULL;
//   ma_device_info* pCaptureDevices = NULL;
//   ma_uint32 nPlaybackDeviceCount = 0;
//   ma_uint32 nCaptureDeviceCount = 0;
//   ma_result result = ma_context_get_devices(&context, &pPlaybackDevices, &nPlaybackDeviceCount, &pCaptureDevices, &nCaptureDeviceCount);
//   if (result != MA_SUCCESS)
//   {
//     printf("[FFT] Failed to enumerate audio devices: %d\n", result);
//     return;
//   }

//   pEnumerationFunction(true, "<default device>", NULL, pUserContext);
//   for (ma_uint32 i = 0; i < nCaptureDeviceCount; i++)
//   {
//     pEnumerationFunction(true, pCaptureDevices[i].name, &pCaptureDevices[i].id, pUserContext);
//   }
//   if (ma_is_loopback_supported(context.backend))
//   {
//     pEnumerationFunction(false, "<default device>", NULL, pUserContext);
//     for (ma_uint32 i = 0; i < nPlaybackDeviceCount; i++)
//     {
//       pEnumerationFunction(false, pPlaybackDevices[i].name, &pPlaybackDevices[i].id, pUserContext);
//     }
//   }
// }

void miniaudioLogCallback(void *userData, ma_uint32 level, const char *message)
{
    (void)userData;
    return;
}

// bool FFT_Create()
// {
//   bCreated = false;
//   ma_context_config context_config = ma_context_config_init();
//   ma_log log;
//   ma_log_init(NULL, &log);
//   ma_log_register_callback(&log, ma_log_callback_init(miniaudioLogCallback, NULL));

//   context_config.pLog = &log;
//   ma_result result = ma_context_init(NULL, 0, &context_config, &context);
//   if (result != MA_SUCCESS)
//   {
//     printf("[FFT] Failed to initialize context: %d", result);
//     return false;
//   }

//   printf("[FFT] MAL context initialized, backend is '%s'\n", ma_get_backend_name(context.backend));
//   bCreated = true;
//   return true;
// }

bool FFT_Destroy()
{
  if (!bCreated)
  {
    return false;
  }

  ma_context_uninit(&context);

  bCreated = false;

  return true;
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
    printf( "[FFT] Failed to initialize context: %d", result );
    return false;
  }

  printf( "[FFT] MAL context initialized, backend is '%s'\n", ma_get_backend_name( context.backend ) );

  ma_device_id* TargetDevice = NULL;

  ma_device_info* pPlaybackDeviceInfos;
  ma_uint32 playbackDeviceCount;
  ma_device_info* pCaptureDeviceInfos;
  ma_uint32 captureDeviceCount;
  result = ma_context_get_devices(&context, &pPlaybackDeviceInfos, &playbackDeviceCount, &pCaptureDeviceInfos, &captureDeviceCount);
  if (result != MA_SUCCESS) {
      printf("Failed to retrieve device information.\n");
      return -3;
  }

  printf("Playback Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < playbackDeviceCount; ++iDevice) {
      printf("    %u: %s\n", iDevice, pPlaybackDeviceInfos[iDevice].name);
  }
  
  printf("\n");

  printf("Capture Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < captureDeviceCount; ++iDevice) {
      printf("    %u: %s\n", iDevice, pCaptureDeviceInfos[iDevice].name);
  }

  // if(strlen(CaptureDeviceSearchString) > 0) {
  //   if(CapturePlaybackDevices) {
  //     for (ma_uint32 iDevice = 0; iDevice < playbackDeviceCount; ++iDevice) {
  //       std::string DeviceName = pPlaybackDeviceInfos[iDevice].name;
  //       if(DeviceName.find(CaptureDeviceSearchString) != std::string::npos ){
  //         TargetDevice = &pPlaybackDeviceInfos[iDevice].id;
  //         break;
  //       }
  //     }
  //   } else {
  //     for (ma_uint32 iDevice = 0; iDevice < captureDeviceCount; ++iDevice) {
  //       std::string DeviceName = pCaptureDeviceInfos[iDevice].name;
  //       if(DeviceName.find(CaptureDeviceSearchString) != std::string::npos ){
  //         TargetDevice = &pCaptureDeviceInfos[iDevice].id;
  //         break;
  //       }
  //     }
  //   }
  // }

  printf("\n");

//   bool useLoopback = ma_is_loopback_supported(context.backend) && !pSettings->bUseRecordingDevice;
//   ma_device_config config = ma_device_config_init(useLoopback ? ma_device_type_loopback : ma_device_type_capture);
//   config.capture.pDeviceID = (ma_device_id*)pSettings->pDeviceID;
//   config.capture.format = ma_format_f32;
//   config.capture.channels = 2;
//   config.sampleRate = 44100;
//   config.dataCallback = OnReceiveFrames;
//   config.pUserData = NULL;

  bool useLoopback = ma_is_loopback_supported(context.backend);
  ma_device_config config = ma_device_config_init(useLoopback ? ma_device_type_loopback : ma_device_type_capture);

  // ma_device_config config = ma_device_config_init( CapturePlaybackDevices ? ma_device_type_loopback : ma_device_type_capture );
  config.capture.pDeviceID = TargetDevice;
  config.capture.format = ma_format_f32;
  config.capture.channels = 2;
  config.sampleRate = 44100;
  config.dataCallback = OnReceiveFrames;
  config.pUserData = NULL;

  result = ma_device_init( &context, &config, &captureDevice );
  if ( result != MA_SUCCESS )
  {
    ma_context_uninit( &context );
    printf( "[FFT] Failed to initialize capture device: %d\n", result );
    return false;
  }

  result = ma_device_start( &captureDevice );
  if ( result != MA_SUCCESS )
  {
    ma_device_uninit( &captureDevice );
    ma_context_uninit( &context );
    printf( "[FFT] Failed to start capture device: %d\n", result );
    return false;
  }

  printf( "[FFT] Capturing %s\n", captureDevice.capture.name );

  return true;
}

// bool FFT_Open(FFT_Settings* pSettings)
// {
//   if (!bCreated)
//   {
//     return false;
//   }

//   memset(sampleBuf, 0, sizeof(float) * FFT_SIZE * 2);

//   fftcfg = kiss_fftr_alloc(FFT_SIZE * 2, false, NULL, NULL);

//   ma_device_info* pPlaybackDeviceInfos;
//   ma_uint32 playbackDeviceCount;
//   ma_device_info* pCaptureDeviceInfos;
//   ma_uint32 captureDeviceCount;
//   ma_result result = ma_context_get_devices(&context, &pPlaybackDeviceInfos, &playbackDeviceCount, &pCaptureDeviceInfos, &captureDeviceCount);
//   if (result != MA_SUCCESS) {
//       printf("Failed to retrieve device information.\n");
//       return false;
//   }

//   printf("Playback Devices\n");
//   for (ma_uint32 iDevice = 0; iDevice < playbackDeviceCount; ++iDevice) {
//       printf("    %u: %s\n", iDevice, pPlaybackDeviceInfos[iDevice].name);
//   }

//   printf("\n");

//   printf("Capture Devices\n");
//   for (ma_uint32 iDevice = 0; iDevice < captureDeviceCount; ++iDevice) {
//       printf("    %u: %s\n", iDevice, pCaptureDeviceInfos[iDevice].name);
//   }

//   config.capture.pDeviceID = (ma_device_id*)pSettings->pDeviceID;
//   config.capture.format = ma_format_f32;
//   config.capture.channels = 2;
//   config.sampleRate = 44100;
//   config.dataCallback = OnReceiveFrames;
//   config.pUserData = NULL;

//   result = ma_device_init(&context, &config, &captureDevice);
//   if (result != MA_SUCCESS)
//   {
//     ma_context_uninit(&context);
//     printf("[FFT] Failed to initialize capture device: %d\n", result);
//     return false;
//   }

//   printf("[FFT] Selected capture device: %s\n", captureDevice.capture.name);

//   result = ma_device_start(&captureDevice);
//   if (result != MA_SUCCESS)
//   {
//     ma_device_uninit(&captureDevice);
//     ma_context_uninit(&context);
//     printf("[FFT] Failed to start capture device: %d\n", result);
//     return false;
//   }

//   return true;
// }

void FFT_Close()
{
  if (!bCreated)
  {
    return;
  }

  ma_device_stop(&captureDevice);

  ma_device_uninit(&captureDevice);

  kiss_fft_free(fftcfg);
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
    float fFFTSmoothingFactor = 0.9;
    for ( int i = 0; i < FFT_SIZE; i++ )
    {
      _samples[i] = _samples[i] * fFFTSmoothingFactor + (1 - fFFTSmoothingFactor) * _samples[i];
    }
  } else {
    for (int i = 0; i < FFT_SIZE; i++)
    {
      static const float scaling = 1.0f / (float)FFT_SIZE;
      _samples[i] = 2.0f * sqrtf(out[i].r * out[i].r + out[i].i * out[i].i) * scaling * fAmplification;
    }
  }

    return true;
}

double tic_api_fft(tic_mem* memory, s32 freq)
{
  return fftData[freq];
}
