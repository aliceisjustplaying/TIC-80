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

// mantratonic normalization
double tic_api_fft2(tic_mem* memory, s32 freq, bool dummy, double dummy2)
{
    static float fft_max[FFT_SIZE] = {0}; // Array to store the maximum FFT values
    float fft_current[FFT_SIZE] = {0}; // Array to store the current FFT values
    float fft_normalized[FFT_SIZE] = {0}; // Array to store the normalized FFT values

    u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
    freq = freq * interval;
    freq = fmin(freq, FFT_SIZE);
    freq = fmax(freq, 0);

    static const float scaling = 1.0f / (float)FFT_SIZE;

    // Calculate the current FFT values
    for (int i = 0; i < FFT_SIZE; ++i) {
        float real = fftBuf[i].r;
        float imag = fftBuf[i].i;
        fft_current[i] = 2.0f * sqrtf(real * real + imag * imag) * scaling;
    }

    // Update the maximum FFT values
    for (int i = 0; i < FFT_SIZE; ++i) {
        if (fft_current[i] > fft_max[i]) {
            fft_max[i] = fft_current[i];
        }
    }

    // Normalize the FFT values
    for (int i = 0; i < FFT_SIZE; ++i) {
        fft_normalized[i] = fft_current[i] / fft_max[i];
    }

    // Calculate the average normalized FFT value for the specified frequency range
    float res = 0.0f;
    for (int i = freq; i < freq + interval; ++i) {
        res += fft_normalized[i];
    }
    res /= interval;

    return res;
}

// this has the RMS + glsl algo
// #include <math.h>

// #define LOG10 2.30258509299404568401799145468436421

// static const float epsilon = 1e-6f; // Small epsilon value to avoid divide by zero

// double tic_api_fft2(tic_mem* memory, s32 freq, bool dummy, double dummy2)
// {
//     u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//     freq = freq * interval;
//     freq = fmin(freq, FFT_SIZE - interval);
//     freq = fmax(freq, 0);

//     static const float scaling = 1.0f / (float)FFT_SIZE;
//     static float prevRMS = 0.0f; // Store the previous RMS value
//     static const float smoothingFactor = 0.9f; // Adjust this value to control the smoothing amount

//     // Calculate the RMS of the whole FFT
//     float sumSquares = 0.0f;
//     for (int i = 0; i < FFT_SIZE; ++i) {
//         float real = fftBuf[i].r;
//         float imag = fftBuf[i].i;
//         sumSquares += real * real + imag * imag;
//     }
//     float rms = sqrtf(sumSquares / FFT_SIZE);

//     // Smooth the RMS value
//     float smoothedRMS = smoothingFactor * prevRMS + (1.0f - smoothingFactor) * rms;
//     prevRMS = smoothedRMS; // Update the previous RMS value for the next iteration

//     // Calculate the FFT value for the specified frequency range
//     float res = 0.0f;
//     for (int i = freq; i < freq + interval; ++i) {
//         float real = fftBuf[i].r;
//         float imag = fftBuf[i].i;
//         float magnitude = 2.0f * sqrtf(real * real + imag * imag) * scaling;
//         res += magnitude / smoothedRMS; // Divide by the smoothed RMS value
//     }

//     // Apply logarithmic scaling and normalization
//     float x = (float)freq / (float)(FFT_SIZE - interval);
//     float xt = exp2f(((1.0f - x) * -9.0f + x * -1.0f)); // 47Hz - 12,000Hz in log scale
//     int index = (int)(xt * (FFT_SIZE - 1));
//     float v = res / interval; // Average the FFT values within the interval

//     v = 20.0f * log10f(v + epsilon) / LOG10; // value to dB
//     v += 24.0f * x; // +3dB/oct

//     res = (v + 40.0f) / 40.0f; // Normalize to [0, 1]
//     res = fmax(0.0f, fmin(1.0f, res)); // Clamp to [0, 1]

//     return res;
// }

// working RMS version
// double tic_api_fft2(tic_mem* memory, s32 freq, bool d, double dd)
// {
//     u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//     freq = freq * interval;
//     freq = fmin(freq, FFT_SIZE);
//     freq = fmax(freq, 0);

//     static const float scaling = 1.0f / (float)FFT_SIZE;
//     static float prevRMS = 0.0f; // Store the previous RMS value
//     static const float smoothingFactor = 0.9f; // Adjust this value to control the smoothing amount

//     // Calculate the RMS of the whole FFT
//     float sumSquares = 0.0f;
//     for (int i = 0; i < FFT_SIZE; ++i) {
//         float real = fftBuf[i].r;
//         float imag = fftBuf[i].i;
//         sumSquares += real * real + imag * imag;
//     }
//     float rms = sqrtf(sumSquares / FFT_SIZE);

//     // Smooth the RMS value
//     float smoothedRMS = smoothingFactor * prevRMS + (1.0f - smoothingFactor) * rms;
//     prevRMS = smoothedRMS; // Update the previous RMS value for the next iteration

//     // Calculate the FFT value for the specified frequency range
//     float res = 0.0f;
//     for (int i = freq; i < freq + interval; ++i) {
//         float real = fftBuf[i].r;
//         float imag = fftBuf[i].i;
//         float magnitude = 2.0f * sqrtf(real * real + imag * imag) * scaling;
//         res += magnitude / smoothedRMS; // Divide by the smoothed RMS value
//     }

//     return res;
// }
// static const float epsilon = 1e-6f; // Small epsilon value to avoid divide by zero
// double tic_api_fft2(tic_mem* memory, s32 freq, bool dummy, double dummy2)
// {
//     u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//     freq = freq * interval;
//     freq = fmin(freq, FFT_SIZE - interval);
//     freq = fmax(freq, 0);

//     float res = 0;
//     static const float scaling = 1.0f / (float)FFT_SIZE;

//     for (int i = freq; i < freq + interval; ++i) {
//         float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//         res += val;
//     }

//     // Apply logarithmic scaling and normalization
//     float x = (float)freq / (float)(FFT_SIZE - interval);
//     float xt = exp2f(((1.0f - x) * -9.0f + x * -1.0f)); // 47Hz - 12,000Hz in log scale
//     int index = (int)(xt * (FFT_SIZE - 1));
//     float v = res / interval; // Average the FFT values within the interval

//     v = 20.0f * log10f(v + epsilon) / LOG10; // value to dB
//     v += 24.0f * x; // +3dB/oct

//     res = (v + 40.0f) / 40.0f; // Normalize to [0, 1]
//     res = fmax(0.0f, fmin(1.0f, res)); // Clamp to [0, 1]

//     return res;
// }

// float fPeakSmoothValue = 0.0f;
// float fPeakMinValue = 0.01f;
// float fPeakSmoothing = 0.995f;
// float fftDataSmoothed[FFT_SIZE] = {0};
// // float fAmplification = 1.0f;
// static const float epsilon = 1e-6f; // Small epsilon value to avoid divide by zero

// double tic_api_fft2(tic_mem* memory, s32 freq, bool fPeakNormalize, double smooth)
// {
//     u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//     freq = freq * interval;
//     freq = fmin(freq, FFT_SIZE - interval);
//     freq = fmax(freq, 0);

//     float res = 0;
//     static const float scaling = 1.0f / (float)FFT_SIZE;

//     if (fPeakNormalize) {
//         float peakValue = fPeakMinValue;
//         for (int i = freq; i < freq + interval; ++i) {
//             float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//             if (val > peakValue) peakValue = val;

//             val *= fAmplification;

//             fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + val * (1 - smooth);
//         }
//         if (peakValue > fPeakSmoothValue) {
//             fPeakSmoothValue = peakValue;
//         }
//         if (peakValue < fPeakSmoothValue) {
//             fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
//         }
//         fAmplification = 1.0f / (fPeakSmoothValue + epsilon);
//     } else {
//         for (int i = freq; i < freq + interval; ++i) {
//             float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//             fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + val * (1 - smooth);
//         }
//     }

//     // Apply logarithmic scaling and normalization
//     float x = (float)freq / (float)(FFT_SIZE - interval);
//     float xt = exp2f(((1.0f - x) * -9.0f + x * -1.0f)); // 47Hz - 12,000Hz in log scale
//     int index = (int)(xt * (FFT_SIZE - 1));
//     float v = fftDataSmoothed[index];

//     v = 20.0f * log10f(v + epsilon) / LOG10; // value to dB
//     v += 24.0f * x; // +3dB/oct

//     res = (v + 40.0f) / 40.0f; // Normalize to [0, 1]
//     res = fmax(0.0f, fmin(1.0f, res)); // Clamp to [0, 1]

//     return res;
// }

// float fPeakSmoothValue = 0.0f;
// float fPeakMinValue = 0.01f;
// float fPeakSmoothing = 0.995f;
// float fftDataSmoothed[FFT_SIZE] = {0};
// float peakValues[FFT_SIZE] = {0.0000001}; // Initialize peakValues array with a small value
// float ticValues[FFT_SIZE] = {0};

// double tic_api_fft2(tic_mem* memory, s32 freq, bool normalize, double smooth)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE - interval);
//   freq = fmax(freq, 0);

//   float res = 0;
//   static const float scaling = 1.0f / (float)FFT_SIZE;

//   bool fPeakNormalize = true;
//   if (fPeakNormalize) {
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//       if (val > peakValue) peakValue = val;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         // ticValues[i] = val / peakValues[i];
//         ticValues[i] = val / max(fPeakMinValue, peakValues[i]);
//         val = ticValues[i] * fAmplification;
//       } else {
//         val *= fAmplification;
//       }

//       fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + val * (1 - smooth);
//       res += fftDataSmoothed[i];
//     }
//     if (peakValue > fPeakSmoothValue) {
//       fPeakSmoothValue = peakValue;
//     }
//     if (peakValue < fPeakSmoothValue) {
//       fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
//     }
//     fAmplification = 1.0f / fPeakSmoothValue;
//   } else {
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         // ticValues[i] = val / peakValues[i];
//         ticValues[i] = val / max(fPeakMinValue, peakValues[i]);
//         res += ticValues[i];
//       } else {
//         res += val;
//       }
//     }
//   }

//   return res;
// }

// float fPeakSmoothValue = 0.0f;
// float fPeakMinValue = 0.01f;
// float fPeakSmoothing = 0.995f;
// float fftDataSmoothed[FFT_SIZE] = {0};
// float peakValues[FFT_SIZE] = {0.0000001}; // Initialize peakValues array with a small value
// float ticValues[FFT_SIZE] = {0};
// // float fAmplification = 1.0f;

// double tic_api_fft2(tic_mem* memory, s32 freq, bool normalize, double smooth)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE - interval);
//   freq = fmax(freq, 0);

//   float res = 0;
//   static const float scaling = 1.0f / (float)FFT_SIZE;

//   bool fPeakNormalize = true;

//   if (fPeakNormalize) {
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//       if (val > peakValue) peakValue = val;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         ticValues[i] = val / peakValues[i];
//         fftDataSmoothed[i] = ticValues[i] * fAmplification;
//       } else {
//         fftDataSmoothed[i] = val * fAmplification;
//       }
//     }
//     if (peakValue > fPeakSmoothValue) {
//       fPeakSmoothValue = peakValue;
//     }
//     if (peakValue < fPeakSmoothValue) {
//       fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
//     }
//     fAmplification = 1.0f / fPeakSmoothValue;

//     if (smooth > 0) {
//       for (int i = freq; i < freq + interval; ++i) {
//         fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + (1 - smooth) * fftDataSmoothed[i];
//         res += fftDataSmoothed[i];
//       }
//     } else {
//       for (int i = freq; i < freq + interval; ++i) {
//         res += fftDataSmoothed[i];
//       }
//     }
//   } else {
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         ticValues[i] = val / peakValues[i];
//         res += ticValues[i];
//       } else {
//         res += val;
//       }
//     }
//   }

//   return res;
// }






// float fPeakSmoothValue = 0.0f;
// float fPeakMinValue = 0.01f;
// float fPeakSmoothing = 0.995f;
// float fftDataSmoothed[FFT_SIZE] = {0};
// float peakValues[FFT_SIZE] = {0.0000001}; // Initialize peakValues array with a small value
// float ticValues[FFT_SIZE] = {0};
// float fAmplification = 1.0f;

// float fPeakSmoothValue = 0.0f;
// float fPeakMinValue = 0.01f;
// float fPeakSmoothing = 0.995f;
// float fftDataSmoothed[FFT_SIZE] = {0};
// float peakValues[FFT_SIZE] = {0.0000001}; // Initialize peakValues array with a small value
// float ticValues[FFT_SIZE] = {0};
// // float fAmplification = 1.0f;
// float fFFTSmoothingFactor = 0.9f;

// double tic_api_fft2(tic_mem* memory, s32 freq, bool normalize, double smooth)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE - interval);
//   freq = fmax(freq, 0);

//   float res = 0;
//   static const float scaling = 1.0f / (float)FFT_SIZE;

//   bool fPeakNormalize = true;

//   if (fPeakNormalize) {
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//       if (val > peakValue) peakValue = val;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         ticValues[i] = val / peakValues[i];
//         fftDataSmoothed[i] = ticValues[i] * fAmplification;
//       } else {
//         fftDataSmoothed[i] = val * fAmplification;
//       }
//     }
//     if (peakValue > fPeakSmoothValue) {
//       fPeakSmoothValue = peakValue;
//     }
//     if (peakValue < fPeakSmoothValue) {
//       fPeakSmoothValue = fPeakSmoothValue * fPeakSmoothing + peakValue * (1 - fPeakSmoothing);
//     }
//     fAmplification = 1.0f / fPeakSmoothValue;

//     for (int i = freq; i < freq + interval; ++i) {
//       fftDataSmoothed[i] = fftDataSmoothed[i] * fFFTSmoothingFactor + (1 - fFFTSmoothingFactor) * fftDataSmoothed[i];
//       res += fftDataSmoothed[i];
//     }
//   } else {
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         ticValues[i] = val / peakValues[i];
//         res += ticValues[i];
//       } else {
//         res += val;
//       }
//     }
//   }

//   return res;
// }

// double tic_api_fft2(tic_mem* memory, s32 freq, /* bool fPeakNormalize = true, */ bool normalize/* = false */, double smooth /* = 0.9 */)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE - interval);
//   freq = fmax(freq, 0);

//   float res = 0;
//   static const float scaling = 1.0f / (float)FFT_SIZE;

//   bool fPeakNormalize = true;

//   if (fPeakNormalize) {
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//       if (val > peakValue) peakValue = val;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         ticValues[i] = val / peakValues[i];
//         res += ticValues[i] * fAmplification;
//       } else {
//         res += val * fAmplification;
//       }

//       if (smooth > 0) {
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
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;

//       if (normalize) {
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         ticValues[i] = val / peakValues[i];
//         res += ticValues[i];
//       } else {
//         res += val;
//       }

//       if (smooth > 0) {
//         fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + (1 - smooth) * val;
//         res = fftDataSmoothed[i];
//       }
//     }
//   }

//   return res;
// }


// normalization works, smoothing does not

// float fPeakSmoothValue = 0.0f;
// float fPeakMinValue = 0.01f;
// float fPeakSmoothing = 0.995f;
// float fftDataSmoothed[FFT_SIZE] = {0};
// float peakValues[FFT_SIZE] = {0.0000001}; // Initialize peakValues array with a small value
// float ticValues[FFT_SIZE] = {0};

// double tic_api_fft2(tic_mem* memory, s32 freq, bool normalize/* = false */, double smooth/* = 0.9*/)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE - interval);
//   freq = fmax(freq, 0);

//   float res = 0;
//   static const float scaling = 1.0f / (float)FFT_SIZE;

//   for (int i = freq; i < freq + interval; ++i) {
//     float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;

//     if (normalize) {
//       peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//       ticValues[i] = val / peakValues[i];
//       res += ticValues[i];
//     } else {
//       res += val;
//     }

//     if (smooth > 0) {
//       fftDataSmoothed[i] = fftDataSmoothed[i] * smooth + (1 - smooth) * val;
//       res = fftDataSmoothed[i];
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



// float fftDataSmoothed[FFT_SIZE] = {0};
// float peakValues[FFT_SIZE] = {0.0000001}; // Initialize peakValues array with a small value
// float ticValues[FFT_SIZE] = {0};

// double tic_api_fft2(tic_mem* memory, s32 freq, bool normalize /* = false */, double smooth /* = 0.9*/)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE - interval);
//   freq = fmax(freq, 0);

//   float res = 0;

//   bool fPeakNormalize = true;

//   if (fPeakNormalize) {
//     printf("peakn: ON\n");
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i);
//       if (val > peakValue) peakValue = val;

//       if (normalize) {
//         printf("normalize: ON\n");
//         peakValues[i] = fmax(peakValues[i] * 0.995f, val);
//         ticValues[i] = val / peakValues[i];
//         res += ticValues[i] * fAmplification;
//       } else {
//         printf("normalize: OFF\n");
//         res += val * fAmplification;
//       }

//       if (smooth > 0) {
//         printf("SMOOTH: on\n");
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
//     printf("original FFT function branch!!!\n");
//     static const float scaling = 1.0f / (float)FFT_SIZE;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0 * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//       res += val;
//     }
//   }

//   return res;
// }


// float fPeakSmoothValue = 0.0f;
// float fPeakMinValue = 0.01f;
// float fPeakSmoothing = 0.995f;
// float fftDataSmoothed[FFT_SIZE] = {0};

// double tic_api_fft2(tic_mem* memory, s32 freq, bool normalize /* = false */, double smooth /* = 0.9 */)
// {
//   u32 interval = FFT_SIZE / 256 / 2; // the 2 is to discard super high frequencies, they suck
//   freq = freq * interval;
//   freq = fmin(freq, FFT_SIZE);
//   freq = fmax(freq, 0);

//   float res = 0;

//   if (normalize) {
//     // printf("normalized\n");
//     float peakValue = fPeakMinValue;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0f * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i);
//       if (val > peakValue) peakValue = val;
//       res += val * fAmplification;

//       if (smooth > 0) {
//         // printf("smoothed\n");
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
//     static const float scaling = 1.0f / (float)FFT_SIZE;
//     for (int i = freq; i < freq + interval; ++i) {
//       float val = 2.0 * sqrtf(fftBuf[i].r * fftBuf[i].r + fftBuf[i].i * fftBuf[i].i) * scaling;
//       res += val;
//     }
//   }

//   return res;
// }
