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

  ma_device_info* pPlaybackInfos;
  ma_uint32 playbackCount;
  ma_device_info* pCaptureInfos;
  ma_uint32 captureCount;
  result = ma_context_get_devices(&context, &pPlaybackInfos, &playbackCount, &pCaptureInfos, &captureCount);
  if (result != MA_SUCCESS) {
      FFT_DebugLog(FFT_LOG_ERROR, "Failed to retrieve device information.\n");
      FFT_DebugLog(FFT_LOG_ERROR, "Error: %s\n", ma_result_description(result));
      return;
  }

  FFT_DebugLog(FFT_LOG_INFO, "Playback Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < playbackCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pPlaybackInfos[iDevice].name);
  }
  
  printf("\n");

  FFT_DebugLog(FFT_LOG_INFO, "Capture Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < captureCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pCaptureInfos[iDevice].name);
  }

  printf("\n");

  return;
}

void print_device_id(ma_device_id id, ma_backend backend) {
    switch (backend) {
        case ma_backend_wasapi:
            wprintf(L"WASAPI ID: %ls\n", id.wasapi);
            break;
        case ma_backend_dsound:
            printf("DirectSound GUID: ");
            for (int i = 0; i < 16; i++) {
                printf("%02X", id.dsound[i]);
                if (i < 15) printf("-");
            }
            printf("\n");
            break;
        case ma_backend_winmm:
            printf("WinMM Device ID: %u\n", id.winmm);
            break;
        case ma_backend_coreaudio:
            printf("Core Audio Device Name: %s\n", id.coreaudio);
            break;
        case ma_backend_sndio:
            printf("sndio Device Identifier: %s\n", id.sndio);
            break;
        case ma_backend_audio4:
            printf("Audio4 Device Path: %s\n", id.audio4);
            break;
        case ma_backend_oss:
            printf("OSS Device Path: %s\n", id.oss);
            break;
        case ma_backend_pulseaudio:
            printf("PulseAudio Device Name: %s\n", id.pulse);
            break;
        case ma_backend_alsa:
            printf("ALSA Device Name: %s\n", id.alsa);
            break;
        case ma_backend_jack:
            printf("JACK Device ID: %d\n", id.jack);
            break;
        case ma_backend_aaudio:
            printf("AAudio Device ID: %d\n", id.aaudio);
            break;
        case ma_backend_opensl:
            printf("OpenSL ES Device ID: %u\n", id.opensl);
            break;
        case ma_backend_webaudio:
            printf("Web Audio Device ID: %s\n", id.webaudio);
            break;
        case ma_backend_custom:
            printf("Custom Backend: Integer ID: %d, String: %s, Pointer: %p\n", id.custom.i, id.custom.s, id.custom.p);
            break;
        case ma_backend_null:
            printf("Null Backend Device ID: %d\n", id.nullbackend);
            break;
        default:
            printf("Unknown backend\n");
    }
}


bool FFT_Open(const char* CaptureDeviceSearchString)
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

  ma_device_info* pPlaybackInfos;
  ma_uint32 playbackCount;
  ma_device_info* pCaptureInfos;
  ma_uint32 captureCount;
  result = ma_context_get_devices(&context, &pPlaybackInfos, &playbackCount, &pCaptureInfos, &captureCount);
  if (result != MA_SUCCESS) {
      FFT_DebugLog(FFT_LOG_ERROR, "Failed to retrieve device information.\n");
      return false;
  }

  FFT_DebugLog(FFT_LOG_INFO, "Playback Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < playbackCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pPlaybackInfos[iDevice].name);
  }
  
  FFT_DebugLog(FFT_LOG_INFO, "\n");

  FFT_DebugLog(FFT_LOG_INFO, "Capture Devices\n");
  for (ma_uint32 iDevice = 0; iDevice < captureCount; ++iDevice) {
      FFT_DebugLog(FFT_LOG_INFO, "    %u: %s\n", iDevice, pCaptureInfos[iDevice].name);
  }

  FFT_DebugLog(FFT_LOG_INFO, "\n");

  // only available on Windows, thus we default to that
  bool useLoopback = (ma_is_loopback_supported(context.backend));
  FFT_DebugLog(FFT_LOG_INFO, "Miniaudio loopback support: %s, Use loopback: %s\n", ma_is_loopback_supported(context.backend) ? "Yes" : "No", useLoopback ? "Yes" : "No");
  
  ma_device_id* TargetDevice = NULL;
  if(CaptureDeviceSearchString && strlen(CaptureDeviceSearchString) > 0) {
    if(useLoopback) {
      for (ma_uint32 iDevice = 0; iDevice < playbackCount; ++iDevice) {
        char* DeviceName = pPlaybackInfos[iDevice].name;
        if(strstr(DeviceName, CaptureDeviceSearchString) != NULL) {
          FFT_DebugLog(FFT_LOG_INFO, "Using playback device %s for config\n", DeviceName);
          TargetDevice = &pPlaybackInfos[iDevice].id;
          print_device_id(*TargetDevice, context.backend); // Pass the backend enum from your context initialization
          FFT_DebugLog(FFT_LOG_INFO, "Selected Device ID logged above.\n");
          break;
        }
      }
    } else {
      for (ma_uint32 iDevice = 0; iDevice < captureCount; ++iDevice) {
        char* DeviceName = pCaptureInfos[iDevice].name;
        if(strstr(DeviceName, CaptureDeviceSearchString) != NULL) {
          FFT_DebugLog(FFT_LOG_INFO, "Using capture device %s for config\n", DeviceName);
          TargetDevice = &pCaptureInfos[iDevice].id;
          print_device_id(*TargetDevice, context.backend); // Pass the backend enum from your context initialization
          FFT_DebugLog(FFT_LOG_INFO, "Selected Device ID logged above.\n");
          break;
        }
      }
    }
  }

  ma_device_config config = {0};

  config = ma_device_config_init( useLoopback ? ma_device_type_loopback : ma_device_type_capture );
  config.capture.pDeviceID = TargetDevice;
  if (useLoopback)
  {
    // This is a workaround for a miniaudio bug; without it, selecting a different loopback device on Windows won't work
    config.playback.pDeviceID = TargetDevice;
  }
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

  float fFFTSmoothingFactor = 0.6f;
  for ( int i = 0; i < FFT_SIZE; i++ )
  {
    fftSmoothingData[i] = fftSmoothingData[i] * fFFTSmoothingFactor + (1 - fFFTSmoothingFactor) * _samples[i];
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
