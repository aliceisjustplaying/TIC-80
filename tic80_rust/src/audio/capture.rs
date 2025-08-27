use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};

#[derive(Clone, Debug)]
pub struct AudioInfo {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Clone, Debug, Default)]
pub struct AudioCaptureConfig {
    pub device_substr: Option<String>,
    pub sample_rate: Option<u32>,
    pub ring_capacity: usize, // in samples (mono)
}

pub struct AudioCaptureHandle {
    _stream: Stream, // keep alive
    pub info: AudioInfo,
    pub pushed_count: Arc<AtomicU64>,
    pub overflow_count: Arc<AtomicU64>,
    pub ring_capacity: usize,
}

#[must_use]
pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    let mut out = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for d in devices {
            if let Ok(name) = d.name() {
                out.push(name);
            }
        }
    }
    out
}

fn pick_device(substr: Option<&str>) -> Option<Device> {
    let host = cpal::default_host();
    if let Some(s) = substr {
        if let Ok(devices) = host.input_devices() {
            for d in devices {
                if let Ok(name) = d.name() {
                    if name.to_lowercase().contains(&s.to_lowercase()) {
                        return Some(d);
                    }
                }
            }
        }
    }
    host.default_input_device()
}

#[must_use]
pub const fn default_ring_capacity() -> usize {
    // Shared buffer sized to max(2*FFT_SIZE, VQT_FFT_SIZE) from C impl: 8192 samples.
    8192
}

/// Start audio capture from the selected device and return the handle and consumer.
///
/// Errors
/// Returns an error if no device is available, device querying fails, or the
/// stream fails to build or start.
#[allow(
    clippy::needless_pass_by_value,
    clippy::missing_errors_doc,
    clippy::too_many_lines
)]
pub fn start_capture(
    cfg: AudioCaptureConfig,
) -> anyhow::Result<(AudioCaptureHandle, Consumer<f32>)> {
    let device = pick_device(cfg.device_substr.as_deref())
        .ok_or_else(|| anyhow::anyhow!("No input device available"))?;
    let device_name = device.name().unwrap_or_else(|_| "<unknown>".to_string());

    // Choose a supported config close to the requested one
    let supported = device
        .supported_input_configs()
        .map_err(|e| anyhow::anyhow!("Failed to get supported configs: {e}"))?;
    let req_rate = cfg.sample_rate.unwrap_or(44_100);
    let mut best: Option<(u32, _)> = None;
    for sc in supported {
        let min = sc.min_sample_rate().0;
        let max = sc.max_sample_rate().0;
        let cand = if req_rate < min {
            min
        } else if req_rate > max {
            max
        } else {
            req_rate
        };
        let diff = cand.abs_diff(req_rate);
        match &mut best {
            Some((best_rate, best_sc)) => {
                let best_diff = (*best_rate).abs_diff(req_rate);
                if diff < best_diff {
                    *best_rate = cand;
                    *best_sc = sc;
                }
            }
            None => best = Some((cand, sc)),
        }
    }
    // Use best found or default
    let stream_cfg = if let Some((rate, sc)) = best {
        if rate != req_rate {
            println!(
                "Audio: requested {} Hz; selecting nearest supported {} Hz, {:?}, {} ch",
                req_rate,
                rate,
                sc.sample_format(),
                sc.channels()
            );
        }
        sc.with_sample_rate(cpal::SampleRate(rate))
    } else {
        let def = device
            .default_input_config()
            .map_err(|e| anyhow::anyhow!("No default input config: {e}"))?;
        println!(
            "Audio: falling back to device default: {} Hz, {:?} format, {} ch",
            def.sample_rate().0,
            def.sample_format(),
            def.channels()
        );
        def
    };

    let sample_format = stream_cfg.sample_format();
    let cfg_fixed: StreamConfig = stream_cfg.into();
    let channels = cfg_fixed.channels;
    let sample_rate = cfg_fixed.sample_rate.0;

    // Ring buffer (mono samples)
    let cap = if cfg.ring_capacity == 0 {
        default_ring_capacity()
    } else {
        cfg.ring_capacity
    };
    let (prod, cons) = RingBuffer::<f32>::new(cap);
    let prod = Arc::new(parking_lot::Mutex::new(prod));
    let pushed = Arc::new(AtomicU64::new(0));
    let overflows = Arc::new(AtomicU64::new(0));

    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(
            &device,
            &cfg_fixed,
            channels,
            prod,
            pushed.clone(),
            overflows.clone(),
        )?,
        SampleFormat::F64 => build_stream::<f64>(
            &device,
            &cfg_fixed,
            channels,
            prod,
            pushed.clone(),
            overflows.clone(),
        )?,
        SampleFormat::I16 => build_stream::<i16>(
            &device,
            &cfg_fixed,
            channels,
            prod,
            pushed.clone(),
            overflows.clone(),
        )?,
        SampleFormat::I32 => build_stream::<i32>(
            &device,
            &cfg_fixed,
            channels,
            prod,
            pushed.clone(),
            overflows.clone(),
        )?,
        SampleFormat::U8 => build_stream::<u8>(
            &device,
            &cfg_fixed,
            channels,
            prod,
            pushed.clone(),
            overflows.clone(),
        )?,
        SampleFormat::U16 => build_stream::<u16>(
            &device,
            &cfg_fixed,
            channels,
            prod,
            pushed.clone(),
            overflows.clone(),
        )?,
        other => return Err(anyhow::anyhow!("Unsupported sample format: {other:?}")),
    };

    stream.play()?;

    let info = AudioInfo {
        device_name,
        sample_rate,
        channels,
    };

    Ok((
        AudioCaptureHandle {
            _stream: stream,
            info,
            ring_capacity: cap,
            pushed_count: pushed,
            overflow_count: overflows,
        },
        cons,
    ))
}

fn build_stream<T>(
    device: &Device,
    cfg: &StreamConfig,
    channels: u16,
    prod: Arc<parking_lot::Mutex<Producer<f32>>>,
    pushed: Arc<AtomicU64>,
    overflows: Arc<AtomicU64>,
) -> anyhow::Result<Stream>
where
    T: Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let err_fn = |e| eprintln!("Audio input error: {e}");
    let ch = channels as usize;
    let stream = device.build_input_stream(
        cfg,
        move |data: &[T], _| {
            // Downmix to mono and push into ring buffer.
            // Note: Producer is used only from this audio callback thread.
            // The consumer lives on the main/tick thread. We wrap the producer
            // in a Mutex solely to satisfy Send/Sync constraints for sharing
            // into the callback; there is no contention in practice.
            let mut p = prod.lock();
            for frame in data.chunks_exact(ch) {
                let mut mono = 0.0f32;
                for &s in frame {
                    let v: f32 = s.to_sample();
                    mono += v;
                }
                mono /= ch as f32;
                // Drop if full; consumer will catch up.
                match p.push(mono) {
                    Ok(()) => {
                        pushed.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        overflows.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        },
        err_fn,
        None,
    )?;
    Ok(stream)
}
