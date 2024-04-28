#define MINIAUDIO_IMPLEMENTATION
#include "fft.h"
#include "miniaudio.h"
#include "tic80_types.h"
#include "api.h"

#include "kiss_fft.h"
#include "kiss_fftr.h"
#include <stdio.h>
#include <stdbool.h>
#include <memory.h>
#include <stdarg.h> // For va_list
#include <time.h>

//////////////////////////////////////////////////////////////////////////

#define FFT_DEBUG

FFT_LogLevel g_currentLogLevel = FFT_LOG_DEBUG;

void FFT_DebugLog(FFT_LogLevel level, const char* format, ...)
{
#ifdef FFT_DEBUG
    if (level <= g_currentLogLevel) {
         // Get current time
        if (level == FFT_LOG_TRACE) {
            time_t now = time(NULL);
            struct tm *tm_now = localtime(&now);
            char time_str[20]; // ISO 8601 format requires 19 characters + null terminator
            strftime(time_str, sizeof(time_str), "%Y-%m-%dT%H:%M:%S", tm_now);

            // Print time and log level
            printf("%s ", time_str); // Print the time
        }
        va_list args;
        va_start(args, format);
        switch(level) {
            case FFT_LOG_TRACE:
                printf("[FFT TRACE]: ");
                break;
            case FFT_LOG_DEBUG:
                printf("[FFT DEBUG]: ");
                break;
            case FFT_LOG_INFO:
                printf("[FFT INFO]: ");
                break;
            case FFT_LOG_WARNING:
                printf("[FFT WARNING]: ");
                break;
            case FFT_LOG_ERROR:
                printf("[FFT ERROR]: ");
                break;
            case FFT_LOG_FATAL:
                printf("[FFT FATAL]: ");
                break;
            case FFT_LOG_OFF:
                // Optionally handle the OFF level by returning early or similar
                return;
        }
        vprintf(format, args);
        va_end(args);
        }
#endif
}


//////////////////////////////////////////////////////////////////////////

kiss_fftr_cfg fftcfg;
ma_context context;
ma_device captureDevice;
float sampleBuf[FFT_SIZE * 2];
float fAmplification = 1.0f;
kiss_fft_cpx fftBuf[FFT_SIZE + 1];

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

    kiss_fftr(fftcfg, sampleBuf, fftBuf);
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
  config.sampleRate = 44100;
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

  return true;
}

void FFT_Close()
{
  ma_device_stop(&captureDevice);
  ma_device_uninit(&captureDevice);
  ma_context_uninit(&context);
  kiss_fft_free(fftcfg);
}

//////////////////////////////////////////////////////////////////////////

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
