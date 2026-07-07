//! Native voice session: WASAPI (shared mode, via cpal) → resample →
//! Vosk on-device recognizer → the flowly-align engine → events to the UI.
//!
//! Shared-mode capture is the microphone "pass-through" story: Windows mixes
//! the mic for every app that asks, so Zoom/Teams/OBS keep the microphone
//! while Flowly listens. Flowly never opens the device exclusively.
//!
//! All recognition is on-device (bundled Vosk model); audio never leaves the
//! machine and is never written to disk.

use flowly_align::{Aligner, AlignerConfig};
use flowly_asr::{HypWord, Hypothesis};
use flowly_script::CompiledScript;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

static MODEL: OnceLock<vosk::Model> = OnceLock::new();

pub enum Cmd {
    Jump(usize),
    Stop,
}

pub struct Session {
    pub cmd_tx: Sender<Cmd>,
    pub stop: Arc<AtomicBool>,
}

pub type SessionSlot = Mutex<Option<Session>>;

fn model_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource dir: {e}"))?
        .join("model");
    if dir.join("conf").exists() || dir.join("am").exists() {
        Ok(dir)
    } else {
        Err(format!("speech model not found at {}", dir.display()))
    }
}

pub fn preload_model(app: &AppHandle) -> Result<(), String> {
    if MODEL.get().is_some() {
        return Ok(());
    }
    vosk::set_log_level(vosk::LogLevel::Error);
    let dir = model_dir(app)?;
    let model = vosk::Model::new(dir.to_string_lossy().as_ref())
        .ok_or_else(|| "couldn't load the speech model".to_string())?;
    let _ = MODEL.set(model);
    Ok(())
}

pub fn list_mics() -> Vec<String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let mut names = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for d in devices {
            if let Ok(name) = d.name() {
                names.push(name);
            }
        }
    }
    names
}

pub fn start(app: AppHandle, script_body: String, mic: Option<String>) -> Result<Session, String> {
    preload_model(&app)?;
    let stop = Arc::new(AtomicBool::new(false));
    let (cmd_tx, cmd_rx) = channel::<Cmd>();
    let stop2 = stop.clone();

    std::thread::Builder::new()
        .name("flowly-voice".into())
        .spawn(move || {
            if let Err(e) = run_session(app.clone(), &script_body, mic, stop2, cmd_rx) {
                let _ = app.emit("engine-error", e);
            }
        })
        .map_err(|e| e.to_string())?;

    Ok(Session { cmd_tx, stop })
}

fn run_session(
    app: AppHandle,
    script_body: &str,
    mic: Option<String>,
    stop: Arc<AtomicBool>,
    cmd_rx: Receiver<Cmd>,
) -> Result<(), String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let script = CompiledScript::compile(script_body);
    let mut aligner = Aligner::new(&script, AlignerConfig::default());

    let model = MODEL.get().expect("preloaded");
    let mut rec = vosk::Recognizer::new(model, 16_000.0)
        .ok_or_else(|| "couldn't create recognizer".to_string())?;
    rec.set_words(true);

    // WASAPI shared mode is cpal's default on Windows: the mic keeps flowing
    // to every other app (Zoom/Teams) for the whole session.
    let host = cpal::default_host();
    let device = match &mic {
        Some(name) => host
            .input_devices()
            .map_err(|e| e.to_string())?
            .find(|d| d.name().map(|n| &n == name).unwrap_or(false))
            .ok_or_else(|| format!("microphone {name:?} not found"))?,
        None => host
            .default_input_device()
            .ok_or_else(|| "no microphone found — check Windows privacy settings".to_string())?,
    };
    let config = device.default_input_config().map_err(|e| {
        format!(
            "couldn't open the microphone ({e}). If it never prompts, allow desktop apps to \
             use the microphone in Windows Settings → Privacy & security → Microphone."
        )
    })?;
    let in_rate = config.sample_rate().0 as f64;
    let channels = config.channels() as usize;

    let (audio_tx, audio_rx) = channel::<Vec<f32>>();
    let err_fn = |e| eprintln!("audio stream error: {e}");
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                let _ = audio_tx.send(data.to_vec());
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                let _ = audio_tx.send(data.iter().map(|&s| s as f32 / 32768.0).collect());
            },
            err_fn,
            None,
        ),
        other => return Err(format!("unsupported sample format {other:?}")),
    }
    .map_err(|e| format!("couldn't start the microphone stream: {e}"))?;
    stream.play().map_err(|e| e.to_string())?;

    let started = Instant::now();
    let now_ms = |t0: Instant| t0.elapsed().as_millis() as u64;
    let mut mono_16k = Resampler::new(in_rate, channels);
    let mut pending: Vec<i16> = Vec::with_capacity(3200);
    let mut last_partial = String::new();
    let mut utter_start_ms: u64 = 0;
    let mut last_tick = Instant::now();

    let emit = |app: &AppHandle, events: &[flowly_align::AlignerEvent]| {
        if !events.is_empty() {
            let _ = app.emit("engine-event", serde_json::to_value(events).unwrap_or_default());
        }
    };

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Cmd::Stop => {
                    stop.store(true, Ordering::Relaxed);
                }
                Cmd::Jump(idx) => {
                    let events = aligner.jump_to(idx, now_ms(started));
                    emit(&app, &events);
                    rec.reset();
                    last_partial.clear();
                    utter_start_ms = 0;
                }
            }
        }
        if stop.load(Ordering::Relaxed) {
            break;
        }

        match audio_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(chunk) => {
                mono_16k.push(&chunk, &mut pending);
                // Feed the recognizer in ~100 ms slabs.
                if pending.len() >= 1600 {
                    let now = now_ms(started);
                    if utter_start_ms == 0 {
                        utter_start_ms = now;
                    }
                    let state = rec.accept_waveform(&pending);
                    pending.clear();
                    match state {
                        Ok(vosk::DecodingState::Finalized) => {
                            let hyp = final_hypothesis(&mut rec, now);
                            last_partial.clear();
                            utter_start_ms = 0;
                            if let Some(h) = hyp {
                                emit(&app, &aligner.feed(&h, now));
                            }
                        }
                        Ok(vosk::DecodingState::Running) => {
                            let partial = rec.partial_result().partial.to_string();
                            if !partial.is_empty() && partial != last_partial {
                                let h = partial_hypothesis(&partial, utter_start_ms, now);
                                last_partial = partial;
                                emit(&app, &aligner.feed(&h, now));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if last_tick.elapsed() >= Duration::from_millis(400) {
            last_tick = Instant::now();
            let events = aligner.tick(now_ms(started));
            emit(&app, &events);
        }
    }
    drop(stream);
    Ok(())
}

fn partial_hypothesis(partial: &str, utter_start: u64, now: u64) -> Hypothesis {
    let words: Vec<&str> = partial.split_whitespace().collect();
    let n = words.len().max(1) as u64;
    let span = (now.saturating_sub(utter_start)).max(n * 120);
    let step = span / n;
    Hypothesis {
        words: words
            .iter()
            .enumerate()
            .map(|(i, w)| HypWord {
                text: (*w).to_string(),
                t_start: utter_start + i as u64 * step,
                t_end: utter_start + (i as u64 + 1) * step,
                stable: false,
            })
            .collect(),
        is_final: false,
        t_emitted: now,
    }
}

fn final_hypothesis(rec: &mut vosk::Recognizer, now: u64) -> Option<Hypothesis> {
    let result = rec.result();
    let single = result.single()?;
    if single.result.is_empty() {
        return None;
    }
    Some(Hypothesis {
        words: single
            .result
            .iter()
            .map(|w| HypWord {
                text: w.word.to_string(),
                t_start: (w.start * 1000.0) as u64,
                t_end: (w.end * 1000.0) as u64,
                stable: true,
            })
            .collect(),
        is_final: true,
        t_emitted: now,
    })
}

/// Downmix to mono and linearly resample to 16 kHz i16.
struct Resampler {
    ratio: f64,
    channels: usize,
    /// Fractional read position into the (virtual) mono input stream.
    pos: f64,
    carry: Vec<f32>,
}

impl Resampler {
    fn new(in_rate: f64, channels: usize) -> Resampler {
        Resampler { ratio: in_rate / 16_000.0, channels, pos: 0.0, carry: Vec::new() }
    }

    fn push(&mut self, interleaved: &[f32], out: &mut Vec<i16>) {
        let ch = self.channels.max(1);
        self.carry.reserve(interleaved.len() / ch);
        for frame in interleaved.chunks_exact(ch) {
            let sum: f32 = frame.iter().sum();
            self.carry.push(sum / ch as f32);
        }
        while (self.pos as usize) + 1 < self.carry.len() {
            let i = self.pos as usize;
            let frac = (self.pos - i as f64) as f32;
            let sample = self.carry[i] * (1.0 - frac) + self.carry[i + 1] * frac;
            out.push((sample.clamp(-1.0, 1.0) * 32767.0) as i16);
            self.pos += self.ratio;
        }
        // Drop consumed samples, keep the tail for interpolation continuity.
        let consumed = (self.pos as usize).min(self.carry.len().saturating_sub(1));
        self.carry.drain(..consumed);
        self.pos -= consumed as f64;
    }
}
