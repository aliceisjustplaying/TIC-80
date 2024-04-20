#define MA_DEBUG_OUTPUT
#define MINIAUDIO_IMPLEMENTATION
#include "fft.h"
#include "miniaudio.h"

#include "kiss_fft.h"
#include "kiss_fftr.h"
#include <stdio.h>
#include <memory.h>
#include "api.h"

//////////////////////////////////////////////////////////////////////////

kiss_fftr_cfg fftcfg;
ma_context context;
ma_device captureDevice;
float sampleBuf[FFT_SIZE * 2];
float fAmplification = 1.0f;
bool bCreated = false;
kiss_fft_cpx fftBuf[FFT_SIZE + 1];
bool bPeakNormalization = true;
float fPeakSmoothValue = 0.0f;
float fPeakMinValue = 0.01f;
float fPeakSmoothing = 0.995f;
float fSmoothFactor = 0.9f;
float fftDataSmoothed[FFT_SIZE] = {0};

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

  kiss_fftr(fftcfg, sampleBuf, fftBuf);
}

void FFT_EnumerateDevices(FFT_ENUMERATE_FUNC pEnumerationFunction, void* pUserContext)
{
  if (!bCreated)
  {
    return;
  }

  ma_device_info* pPlaybackDevices = NULL;
  ma_device_info* pCaptureDevices = NULL;
  ma_uint32 nPlaybackDeviceCount = 0;
  ma_uint32 nCaptureDeviceCount = 0;
  ma_result result = ma_context_get_devices(&context, &pPlaybackDevices, &nPlaybackDeviceCount, &pCaptureDevices, &nCaptureDeviceCount);
  if (result != MA_SUCCESS)
  {
    printf("[FFT] Failed to enumerate audio devices: %d\n", result);
    return;
  }

  pEnumerationFunction(true, "<default device>", NULL, pUserContext);
  for (ma_uint32 i = 0; i < nCaptureDeviceCount; i++)
  {
    pEnumerationFunction(true, pCaptureDevices[i].name, &pCaptureDevices[i].id, pUserContext);
  }
  if (ma_is_loopback_supported(context.backend))
  {
    pEnumerationFunction(false, "<default device>", NULL, pUserContext);
    for (ma_uint32 i = 0; i < nPlaybackDeviceCount; i++)
    {
      pEnumerationFunction(false, pPlaybackDevices[i].name, &pPlaybackDevices[i].id, pUserContext);
    }
  }
}

// source: https://github.com/Chatterino/chatterino2/blob/7c97e6bcc748755ee55447096ebc657be11b4789/src/controllers/sound/MiniaudioBackend.cpp#L28
void miniaudioLogCallback(void *userData, ma_uint32 level, const char *message)
{
    (void)userData;
    return;
    // printf("[FFT] [miniaudio:%p]\n %s", userData, message);
    // switch (level)
    // {
    //     case MA_LOG_LEVEL_DEBUG: {
    //       printf( "miniaudio debug:  %s", message);
    //     }
    //     break;
    //     case MA_LOG_LEVEL_INFO: {
    //         printf( "miniaudio info:   %s", message);
    //     }
    //     break;
    //     case MA_LOG_LEVEL_WARNING: {
    //         printf( "miniaudio warning: %s", message);
    //     }
    //     break;
    //     case MA_LOG_LEVEL_ERROR: {
    //         printf( "miniaudio error:  %s", message);
    //     }
    //     break;
    //     default: {
    //         printf( "miniaudio unknown: %s", message);
    //     }
    //     break;
    // }
}

bool FFT_Create()
{
  bCreated = false;
  ma_context_config context_config = ma_context_config_init();
  ma_log log;
  ma_log_init(NULL, &log);
  ma_log_register_callback(&log, ma_log_callback_init(miniaudioLogCallback, NULL));

  context_config.pLog = &log;
  ma_result result = ma_context_init(NULL, 0, &context_config, &context);
  if (result != MA_SUCCESS)
  {
    printf("[FFT] Failed to initialize context: %d", result);
    return false;
  }

  printf("[FFT] MAL context initialized, backend is '%s'\n", ma_get_backend_name(context.backend));
  bCreated = true;
  return true;
}

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

bool FFT_Open(FFT_Settings* pSettings)
{
  if (!bCreated)
  {
    return false;
  }

  memset(sampleBuf, 0, sizeof(float) * FFT_SIZE * 2);

  fftcfg = kiss_fftr_alloc(FFT_SIZE * 2, false, NULL, NULL);

  bool useLoopback = ma_is_loopback_supported(context.backend) && !pSettings->bUseRecordingDevice;
  ma_device_config config = ma_device_config_init(useLoopback ? ma_device_type_loopback : ma_device_type_capture);
  config.capture.pDeviceID = (ma_device_id*)pSettings->pDeviceID;
  config.capture.format = ma_format_f32;
  config.capture.channels = 2;
  config.sampleRate = 44100;
  config.dataCallback = OnReceiveFrames;
  config.pUserData = NULL;

  ma_result result = ma_device_init(&context, &config, &captureDevice);
  if (result != MA_SUCCESS)
  {
    printf("[FFT] Failed to initialize capture device: %d\n", result);
    return false;
  }

  printf("[FFT] Selected capture device: %s\n", captureDevice.capture.name);

  result = ma_device_start(&captureDevice);
  if (result != MA_SUCCESS)
  {
    ma_device_uninit(&captureDevice);
    printf("[FFT] Failed to start capture device: %d\n", result);
    return false;
  }

  return true;
}

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


float tic_api_fft2(tic_mem* memory, s32 freq, bool normalize, float smooth)
{
  u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
  freq = freq * interval;
  freq = fmin(freq, FFT_SIZE);
  freq = fmax(freq, 0);

  float res = 0;

  if (normalize) {
    printf("normalized\n");
    float peakValue = fPeakMinValue;
    for (int i = freq; i < freq + interval; ++i) {
      float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i);
      if (val > peakValue) peakValue = val;
      res += val * fAmplification;

      if (smooth > 0) {
        printf("smoothed\n");
        fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + (1 - smooth) * val;
        res = fftDataSmoothed[i];
      }
    }
    if (peakValue > fPeakSmoothValue) {
      fPeakSmoothValue = peakValue;
    }
    if (peakValue < fPeakSmoothValue) {
      fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
    }
    fAmplification = 1.0f / fPeakSmoothValue;
  } else {
    static const float scaling = 1.0f / (float)FFT_SIZE;
    for (int i = freq; i < freq + interval; ++i) {
      float val = 2.0 * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
      res += val;
    }
  }

  return res;
}

// double tic_api_fft(tic_mem* memory, s32 freq)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE);
//   freq = fmax(freq, 0);

//   static const float scaling = 1.0f / (float)FFT_SIZE;
//   float res = 0;
//   for (int i = freq; i < freq + interval; ++i) {
//     res += 2.0 * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//   }

//   return res;
// }

// bad
// float tic_api_fft2(tic_mem* memory, s32 freq, bool normalize, float smooth)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE);
//   freq = fmax(freq, 0);

//   float res = 0;

//   if (normalize) {
//     printf("normalized\n");
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i);
//       if (val > peakValue) peakValue = val;
//       res += val * fAmplification;

//       if (smooth > 0) {
//         printf("normalized and smoothed\n");
//         fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + (1 - smooth) * val;
//         res = fftDataSmoothed[i];
//       }
//     }
//     if (peakValue > fPeakSmoothValue) {
//       fPeakSmoothValue = peakValue;
//     }
//     if (peakValue < fPeakSmoothValue) {
//       fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
//     }
//     fAmplification = 1.0f / fPeakSmoothValue;
//   } else {
//     printf("regular\n");
//     static const float scaling = 1.0f / (float)FFT_SIZE;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0 * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling * fAmplification;
//       res += val;

//       if (smooth > 0) {
//         printf("regular and smooth\n");
//         fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + (1 - smooth) * val;
//         res = fftDataSmoothed[i];
//       }
//     }
//   }

//   return res;
// }

// float tic_api_fft2(tic_mem* memory, s32 freq, bool normalize, float smooth)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE);
//   freq = fmax(freq, 0);
//   printf("normalize value: %d\n", normalize);
//   printf("smooth value: %.2f\n", smooth);

//   float res = 0;

//   if (normalize) {
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i);
//       if (val > peakValue) peakValue = val;
//       res += val * fAmplification;
//     }
//     if (peakValue > fPeakSmoothValue) {
//       fPeakSmoothValue = peakValue;
//     }
//     if (peakValue < fPeakSmoothValue) {
//       fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
//     }
//     fAmplification = 1.0f / fPeakSmoothValue;
//   } else {
//     static const float scaling = 1.0f / (float)FFT_SIZE;
//     for (int i = freq; i < freq + interval; ++i) {
//       res += 2.0 * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling * fAmplification;
//     }
//   }

//   return res;
// }

double tic_api_fft(tic_mem* memory, s32 freq)
{
  u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
  freq = freq * interval;
  freq = fmin(freq, FFT_SIZE);
  freq = fmax(freq, 0);

  static const float scaling = 1.0f / (float)FFT_SIZE;
  float res = 0;
  for (int i = freq; i < freq + interval; ++i) {
    res += 2.0 * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
  }

  return res;
}
