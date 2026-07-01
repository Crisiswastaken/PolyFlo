use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, Stream};

use super::{resample, TARGET_SAMPLE_RATE};

#[derive(Clone)]
pub struct AudioBuffer {
    inner: Arc<Mutex<Vec<i16>>>,
    meter_cursor: Arc<Mutex<usize>>,
}

impl AudioBuffer {
    pub fn drain_pcm(&self) -> Vec<u8> {
        let samples = {
            let mut guard = self.inner.lock().unwrap();
            *self.meter_cursor.lock().unwrap() = 0;
            guard.drain(..).collect::<Vec<i16>>()
        };

        let mut bytes = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }

    /// Level from audio captured since the last meter read — tracks live speech.
    pub fn peak_level(&self) -> f32 {
        let guard = self.inner.lock().unwrap();
        let mut cursor = self.meter_cursor.lock().unwrap();
        let start = *cursor;
        let end = guard.len();
        if start >= end {
            return 0.0;
        }
        *cursor = end;
        compute_meter_level(&guard[start..end])
    }

    /// RMS + peak blend over a recent slice (used by tests / fallback).
    pub fn meter_level(&self, sample_window: usize) -> f32 {
        let guard = self.inner.lock().unwrap();
        if guard.is_empty() {
            return 0.0;
        }
        let start = guard.len().saturating_sub(sample_window);
        compute_meter_level(&guard[start..])
    }
}

fn compute_meter_level(slice: &[i16]) -> f32 {
    if slice.is_empty() {
        return 0.0;
    }

    let peak = slice
        .iter()
        .map(|s| s.abs())
        .max()
        .unwrap_or(0) as f32
        / i16::MAX as f32;

    let sum_sq: f64 = slice
        .iter()
        .map(|&s| {
            let n = s as f64 / i16::MAX as f64;
            n * n
        })
        .sum();
    let rms = (sum_sq / slice.len() as f64).sqrt() as f32;

    let raw = peak * 0.35 + rms * 0.65;
    (raw * 9.0).powf(0.55).clamp(0.0, 1.0)
}

/// Owns the cpal stream on a dedicated thread (Stream is !Send).
pub struct AudioCaptureGuard {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl AudioCaptureGuard {
    pub fn start() -> Result<(AudioBuffer, Self), String> {
        let inner = Arc::new(Mutex::new(Vec::<i16>::new()));
        let meter_cursor = Arc::new(Mutex::new(0));
        let buffer = AudioBuffer {
            inner: inner.clone(),
            meter_cursor,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();

        let thread = thread::spawn(move || {
            if let Err(e) = run_capture_loop(inner, stop_thread) {
                tracing::error!("Audio capture thread error: {e}");
            }
        });

        Ok((
            buffer,
            Self {
                stop,
                thread: Some(thread),
            },
        ))
    }
}

impl Drop for AudioCaptureGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn run_capture_loop(buffer: Arc<Mutex<Vec<i16>>>, stop: Arc<AtomicBool>) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No input device available".to_string())?;

    let config = device
        .default_input_config()
        .map_err(|e| e.to_string())?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();

    let stream = match sample_format {
        SampleFormat::I16 => {
            build_stream::<i16>(&device, &config.into(), channels, sample_rate, buffer.clone())?
        }
        SampleFormat::F32 => {
            build_stream::<f32>(&device, &config.into(), channels, sample_rate, buffer.clone())?
        }
        SampleFormat::U16 => {
            build_stream::<u16>(&device, &config.into(), channels, sample_rate, buffer.clone())?
        }
        _ => return Err(format!("Unsupported sample format: {sample_format:?}")),
    };

    stream.play().map_err(|e| e.to_string())?;

    while !stop.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(50));
    }

    drop(stream);
    Ok(())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sample_rate: u32,
    buffer: Arc<Mutex<Vec<i16>>>,
) -> Result<Stream, String>
where
    T: Sample + cpal::SizedSample,
    f32: SampleToF32<T>,
{
    let err_fn = |err| tracing::error!("Audio stream error: {err}");

    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mono: Vec<f32> = data
                    .chunks(channels)
                    .map(|frame| {
                        let sum: f32 = frame
                            .iter()
                            .map(|s| <f32 as SampleToF32<T>>::to_f32(*s))
                            .sum();
                        sum / channels as f32
                    })
                    .collect();

                let resampled = if sample_rate == TARGET_SAMPLE_RATE {
                    mono.iter().map(|&s| f32_to_i16(s)).collect()
                } else {
                    resample::resample_to_16k(&mono, sample_rate)
                };

                if let Ok(mut guard) = buffer.lock() {
                    guard.extend(resampled);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())
}

trait SampleToF32<T> {
    fn to_f32(sample: T) -> f32;
}

impl SampleToF32<i16> for f32 {
    fn to_f32(sample: i16) -> f32 {
        sample as f32 / i16::MAX as f32
    }
}

impl SampleToF32<f32> for f32 {
    fn to_f32(sample: f32) -> f32 {
        sample
    }
}

impl SampleToF32<u16> for f32 {
    fn to_f32(sample: u16) -> f32 {
        (sample as f32 / u16::MAX as f32) * 2.0 - 1.0
    }
}

fn f32_to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * i16::MAX as f32) as i16
}

pub fn test_mic(duration_ms: u64) -> Result<bool, String> {
    let (buffer, _guard) = AudioCaptureGuard::start()?;
    thread::sleep(std::time::Duration::from_millis(duration_ms));
    Ok(buffer.drain_pcm().len() > 320)
}
