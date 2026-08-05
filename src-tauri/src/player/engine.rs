use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

const PREBUFFER_SAMPLES: usize = 48000 / 5;
const MAX_BUFFER_SAMPLES: usize = 48000 * 2 * 10;

pub struct AudioSink {
    pub stream: cpal::Stream,
    pub position_secs: Arc<Mutex<f64>>,
    pub is_paused: Arc<Mutex<bool>>,
    pub stop_flag: Arc<AtomicBool>,
    pub volume: Arc<Mutex<f32>>,
    pub seek_request: Arc<Mutex<Option<f64>>>,
    pub audio_buf: Arc<Mutex<Vec<f32>>>,
    pub device_sample_rate: u32,
    pub device_channels: usize,
}

unsafe impl Send for AudioSink {}
unsafe impl Sync for AudioSink {}

pub struct AudioEngine {
    pub sink: Arc<Mutex<Option<AudioSink>>>,
}

unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            sink: Arc::new(Mutex::new(None)),
        }
    }

    pub fn play(&self, path: &str) -> Result<(), String> {
        if let Ok(mut guard) = self.sink.lock() {
            if let Some(old_sink) = guard.as_ref() {
                old_sink.stop_flag.store(true, Ordering::SeqCst);
            }
            *guard = None;
        }
        let path = path.to_string();
        let sink = self.sink.clone();
        std::thread::spawn(move || {
            if let Err(e) = play_file(&path, sink) {
                log::error!("Playback error: {}", e);
            }
        });
        Ok(())
    }

    pub fn pause(&self) {
        if let Ok(guard) = self.sink.lock() {
            if let Some(sink) = guard.as_ref() {
                *sink.is_paused.lock().unwrap() = true;
                sink.stream.pause().ok();
            }
        }
    }

    pub fn resume(&self) {
        if let Ok(guard) = self.sink.lock() {
            if let Some(sink) = guard.as_ref() {
                *sink.is_paused.lock().unwrap() = false;
                sink.stream.play().ok();
            }
        }
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.sink.lock() {
            if let Some(sink) = guard.as_ref() {
                sink.stop_flag.store(true, Ordering::SeqCst);
            }
            *guard = None;
        }
    }

    pub fn position(&self) -> f64 {
        if let Ok(guard) = self.sink.lock() {
            if let Some(sink) = guard.as_ref() {
                return *sink.position_secs.lock().unwrap();
            }
        }
        0.0
    }

    pub fn set_volume(&self, volume: f32) {
        if let Ok(guard) = self.sink.lock() {
            if let Some(sink) = guard.as_ref() {
                *sink.volume.lock().unwrap() = volume.clamp(0.0, 1.0);
            }
        }
    }

    pub fn seek(&self, position_secs: f64) {
    if let Ok(guard) = self.sink.lock() {
        if let Some(sink) = guard.as_ref() {
            *sink.seek_request.lock().unwrap() = Some(position_secs);
            sink.audio_buf.lock().unwrap().clear();
            *sink.position_secs.lock().unwrap() = position_secs;
        }
    }
}
}

fn play_file(path: &str, sink_handle: Arc<Mutex<Option<AudioSink>>>) -> Result<(), String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| e.to_string())?;

    let mut format = probed.format;
    let track = format.default_track().ok_or("No default track")?;
    let track_id = track.id;
    let file_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let file_channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;

    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("No output device")?;
    let config = device.default_output_config().map_err(|e| e.to_string())?;
    let device_sample_rate = config.sample_rate().0;
    let device_channels = config.channels() as usize;

    // TEMP: remove before ship
    log::info!("File: {}Hz {}ch, Device: {}Hz {}ch", file_sample_rate, file_channels, device_sample_rate, device_channels);

    let needs_resample = file_sample_rate != device_sample_rate;
    let resample_ratio = device_sample_rate as f64 / file_sample_rate as f64;

    let audio_buf: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(MAX_BUFFER_SAMPLES)));
    let audio_buf_decoder = audio_buf.clone();
    let audio_buf_stream = audio_buf.clone();

    let position_secs: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
    let position_writer = position_secs.clone();
    let is_paused: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let is_paused_stream = is_paused.clone();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_decoder = stop_flag.clone();
    let stop_flag_stream = stop_flag.clone();
    let volume: Arc<Mutex<f32>> = Arc::new(Mutex::new(1.0));
    let volume_stream = volume.clone();
    let seek_request: Arc<Mutex<Option<f64>>> = Arc::new(Mutex::new(None));
    let seek_request_decoder = seek_request.clone();

    let chunk_size = 1024usize;
    let mut resampler: Option<SincFixedIn<f32>> = if needs_resample {
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        Some(SincFixedIn::<f32>::new(
            resample_ratio,
            2.0,
            params,
            chunk_size,
            file_channels,
        ).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let decode_thread = std::thread::spawn(move || {
        let mut sample_buf: Option<SampleBuffer<f32>> = None;
        let mut pending: Vec<Vec<f32>> = vec![Vec::new(); file_channels];

        loop {
            if stop_flag_decoder.load(Ordering::SeqCst) {
                break;
            }

            // Handle seek request
            if let Some(seek_pos) = seek_request_decoder.lock().unwrap().take() {
                let _ = format.seek(SeekMode::Accurate, SeekTo::Time {
                    time: Time::from(seek_pos),
                    track_id: None,
                });
                decoder.reset();
                for ch in &mut pending {
                    ch.clear();
                }
            }

            {
                let buf = audio_buf_decoder.lock().unwrap();
                if buf.len() >= MAX_BUFFER_SAMPLES {
                    drop(buf);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
            }

            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(_) => break,
            };

            if packet.track_id() != track_id { continue; }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    if sample_buf.is_none() {
                        let spec = *decoded.spec();
                        let cap = decoded.capacity() as u64;
                        sample_buf = Some(SampleBuffer::<f32>::new(cap, spec));
                    }
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(decoded);
                        let samples = buf.samples();

                        if let Some(resampler) = &mut resampler {
                            for (i, &s) in samples.iter().enumerate() {
                                pending[i % file_channels].push(s);
                            }
                            while pending[0].len() >= chunk_size {
                                let chunk: Vec<&[f32]> = pending.iter()
                                    .map(|ch| &ch[..chunk_size])
                                    .collect();
                                if let Ok(resampled) = resampler.process(&chunk, None) {
                                    let out_len = resampled[0].len();
                                    let mut audio = audio_buf_decoder.lock().unwrap();
                                    if file_channels == 1 && device_channels == 2 {
                                        for i in 0..out_len {
                                            audio.push(resampled[0][i]);
                                            audio.push(resampled[0][i]);
                                        }
                                    } else {
                                        for i in 0..out_len {
                                            for ch in &resampled {
                                                audio.push(ch[i]);
                                            }
                                        }
                                    }
                                }
                                for ch in &mut pending {
                                    ch.drain(..chunk_size);
                                }
                            }
                        } else {
                            let mut audio = audio_buf_decoder.lock().unwrap();
                            if file_channels == 1 && device_channels == 2 {
                                for &s in samples {
                                    audio.push(s);
                                    audio.push(s);
                                }
                            } else {
                                audio.extend_from_slice(samples);
                            }
                        }
                    }
                }
                Err(e) => log::warn!("Decode error: {}", e),
            }
        }
    });

    loop {
        let len = audio_buf.lock().unwrap().len();
        if len >= PREBUFFER_SAMPLES { break; }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            if stop_flag_stream.load(Ordering::SeqCst) {
                data.fill(0.0);
                return;
            }
            if *is_paused_stream.lock().unwrap() {
                data.fill(0.0);
                return;
            }
            let vol = *volume_stream.lock().unwrap();
            let mut buf = audio_buf_stream.lock().unwrap();
            let len = data.len().min(buf.len());
            for (i, sample) in data[..len].iter_mut().enumerate() {
                *sample = buf[i] * vol;
            }
            if len < data.len() {
                data[len..].fill(0.0);
            }
            buf.drain(..len);
            let mut pos = position_writer.lock().unwrap();
            *pos += len as f64 / (device_sample_rate as f64 * device_channels as f64);
        },
        |err| log::error!("Stream error: {}", err),
        None,
    ).map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    *sink_handle.lock().unwrap() = Some(AudioSink {
        stream,
        position_secs,
        is_paused,
        stop_flag,
        volume,
        seek_request,
        audio_buf,
        device_sample_rate,
        device_channels,
    });

    decode_thread.join().ok();
    Ok(())
}