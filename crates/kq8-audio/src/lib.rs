//! Sound output (M3 step 3h-2): our own mixer, and the sound card through
//! `cpal` (the library the user chose, 2026-10-17).
//!
//! The rules (which sound, how loud, which side) are in `kq8-core`
//! `sound.rs`; this crate only does what it is told: start, change the two
//! channel loudnesses, pause, resume, stop. See docs/DESIGN.md M3 3h-2.
//!
//! - Each sound plays at its own file rate; the mixer converts it to the
//!   card's rate by a straight line between neighbouring samples (our
//!   method; `sound.md` rule 18 says any rate is fine).
//! - A mono sound goes to both channels, a stereo one left to left, right
//!   to right, each times its channel loudness (`sound.md` rule 23).
//! - Looping sounds start over at the end with no gap (rule 25); one-shots
//!   are dropped when they end (rule 24).
//! - The sum is clipped to the card's range (our choice; the original left
//!   this to DirectSound).

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use kq8_data::aud::Sound;

/// What the mixer is told to do; `id` names one playing sound.
#[derive(Debug, Clone)]
pub enum Command {
    Start { id: u64, sound: Arc<Sound>, looping: bool, gains: [f32; 2] },
    Gains { id: u64, gains: [f32; 2] },
    Pause { id: u64 },
    Resume { id: u64 },
    Stop { id: u64 },
}

#[derive(Debug)]
struct Voice {
    id: u64,
    sound: Arc<Sound>,
    /// Position in the sound, in its own samples per channel.
    pos: f64,
    /// File samples per card sample.
    step: f64,
    looping: bool,
    paused: bool,
    gains: [f32; 2],
}

impl Voice {
    /// Sample `frame` of channel `ch` (0 or 1), from -1 to 1; past the end
    /// of a one-shot 0, of a loop the start again.
    fn sample(&self, frame: usize, ch: usize) -> f32 {
        let s = &self.sound;
        let n = s.channels as usize;
        let frames = s.frames();
        let frame = if frame >= frames {
            if self.looping && frames > 0 {
                frame % frames
            } else {
                return 0.0;
            }
        } else {
            frame
        };
        f32::from(s.samples[frame * n + ch.min(n - 1)]) / 32_768.0
    }
}

/// Adds up the playing sounds.
#[derive(Debug)]
pub struct Mixer {
    rate: u32,
    voices: Vec<Voice>,
}

impl Mixer {
    /// A mixer for a card running at `rate` samples per second.
    pub fn new(rate: u32) -> Self {
        Mixer { rate, voices: Vec::new() }
    }

    /// Sounds being mixed (paused ones included).
    pub fn voices(&self) -> usize {
        self.voices.len()
    }

    pub fn apply(&mut self, c: Command) {
        let find = |v: &mut Vec<Voice>, id: u64| v.iter().position(|x| x.id == id);
        match c {
            Command::Start { id, sound, looping, gains } => {
                let step = f64::from(sound.rate) / f64::from(self.rate.max(1));
                if sound.frames() > 0 {
                    self.voices.push(Voice { id, sound, pos: 0.0, step, looping, paused: false, gains });
                }
            }
            Command::Gains { id, gains } => {
                if let Some(n) = find(&mut self.voices, id) {
                    self.voices[n].gains = gains;
                }
            }
            Command::Pause { id } | Command::Resume { id } => {
                let pause = matches!(c, Command::Pause { .. });
                if let Some(n) = find(&mut self.voices, id) {
                    self.voices[n].paused = pause;
                }
            }
            Command::Stop { id } => {
                if let Some(n) = find(&mut self.voices, id) {
                    self.voices.remove(n);
                }
            }
        }
    }

    /// Fills `out` (interleaved, `channels` per frame) with the mix. With
    /// one channel, left and right are averaged; past two, the rest are 0.
    pub fn mix(&mut self, out: &mut [f32], channels: usize) {
        out.fill(0.0);
        let channels = channels.max(1);
        let frames = out.len() / channels;
        for v in self.voices.iter_mut().filter(|v| !v.paused) {
            let total = v.sound.frames() as f64;
            for f in 0..frames {
                if !v.looping && v.pos >= total {
                    break;
                }
                let i = v.pos.floor() as usize;
                let t = (v.pos - v.pos.floor()) as f32;
                let mut lr = [0.0f32; 2];
                for (ch, x) in lr.iter_mut().enumerate() {
                    let (a, b) = (v.sample(i, ch), v.sample(i + 1, ch));
                    *x = (a + (b - a) * t) * v.gains[ch];
                }
                let o = &mut out[f * channels..(f + 1) * channels];
                if channels == 1 {
                    o[0] += 0.5 * (lr[0] + lr[1]);
                } else {
                    o[0] += lr[0];
                    o[1] += lr[1];
                }
                v.pos += v.step;
                if v.looping && v.pos >= total {
                    v.pos -= total;
                }
            }
        }
        self.voices.retain(|v| v.looping || v.pos < v.sound.frames() as f64);
        for x in out.iter_mut() {
            *x = x.clamp(-1.0, 1.0);
        }
    }
}

/// The sound card, fed by a [`Mixer`].
pub struct Output {
    _stream: cpal::Stream,
    mixer: Arc<Mutex<Mixer>>,
    pub rate: u32,
    pub channels: u16,
    /// The card's name and sample format, for the log.
    pub description: String,
}

impl Output {
    /// Opens the default sound card at its default settings.
    pub fn open() -> Result<Output, String> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("no default sound output device")?;
        let name = device.description().map(|d| d.name().to_string()).unwrap_or_else(|e| format!("(name unknown: {e})"));
        let config = device.default_output_config().map_err(|e| format!("{name}: {e}"))?;
        let format = config.sample_format();
        let config: cpal::StreamConfig = config.into();
        let (rate, channels) = (config.sample_rate, config.channels);
        let mixer = Arc::new(Mutex::new(Mixer::new(rate)));
        let stream = match format {
            SampleFormat::F32 => build::<f32>(&device, config, mixer.clone()),
            SampleFormat::I16 => build::<i16>(&device, config, mixer.clone()),
            SampleFormat::U16 => build::<u16>(&device, config, mixer.clone()),
            SampleFormat::I32 => build::<i32>(&device, config, mixer.clone()),
            SampleFormat::F64 => build::<f64>(&device, config, mixer.clone()),
            other => Err(format!("sample format {other} not handled")),
        }
        .map_err(|e| format!("{name}: {e}"))?;
        stream.play().map_err(|e| format!("{name}: {e}"))?;
        Ok(Output {
            _stream: stream,
            mixer,
            rate,
            channels,
            description: format!("{name}, {rate} Hz, {channels} channels, {format}"),
        })
    }

    pub fn send(&self, commands: impl IntoIterator<Item = Command>) {
        if let Ok(mut m) = self.mixer.lock() {
            for c in commands {
                m.apply(c);
            }
        }
    }

    /// Sounds the mixer holds now.
    pub fn voices(&self) -> usize {
        self.mixer.lock().map_or(0, |m| m.voices())
    }
}

fn build<T>(device: &cpal::Device, config: cpal::StreamConfig, mixer: Arc<Mutex<Mixer>>) -> Result<cpal::Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let mut scratch: Vec<f32> = Vec::new();
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                scratch.resize(data.len(), 0.0);
                match mixer.lock() {
                    Ok(mut m) => m.mix(&mut scratch, channels),
                    Err(_) => scratch.fill(0.0),
                }
                for (d, s) in data.iter_mut().zip(&scratch) {
                    *d = T::from_sample(*s);
                }
            },
            |e| eprintln!("sound output error: {e}"),
            None,
        )
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_data::aud::Kind;

    fn sound(rate: u32, channels: u16, samples: Vec<i16>) -> Arc<Sound> {
        Arc::new(Sound {
            kind: Kind::Wav { bits: 16 },
            rate,
            channels,
            peak: 0,
            declared_len: 0,
            present_len: 0,
            wraps: 0,
            samples,
        })
    }

    #[test]
    fn mono_goes_to_both_sides_at_their_gains() {
        let mut m = Mixer::new(100);
        m.apply(Command::Start { id: 1, sound: sound(100, 1, vec![16_384; 4]), looping: false, gains: [0.18, 1.0] });
        let mut out = [0.0f32; 4];
        m.mix(&mut out, 2);
        assert!((out[0] - 0.09).abs() < 1e-6 && (out[1] - 0.5).abs() < 1e-6, "{out:?}");
    }

    #[test]
    fn stereo_keeps_its_sides() {
        let mut m = Mixer::new(100);
        m.apply(Command::Start { id: 1, sound: sound(100, 2, vec![16_384, -16_384]), looping: false, gains: [1.0, 1.0] });
        let mut out = [0.0f32; 2];
        m.mix(&mut out, 2);
        assert_eq!(out, [0.5, -0.5]);
    }

    #[test]
    fn rate_conversion_keeps_the_length() {
        // 22,050 Hz into 44,100 Hz: 100 samples last 200 card samples.
        let mut m = Mixer::new(44_100);
        m.apply(Command::Start { id: 1, sound: sound(22_050, 1, vec![1000; 100]), looping: false, gains: [1.0, 1.0] });
        let mut out = vec![0.0f32; 2 * 300];
        m.mix(&mut out, 2);
        let heard = out.chunks(2).filter(|f| f[0] != 0.0).count();
        assert_eq!(heard, 200);
        assert_eq!(m.voices(), 0, "a finished one-shot is dropped");
        // Halfway between two samples: a straight line.
        let mut m = Mixer::new(200);
        m.apply(Command::Start { id: 1, sound: sound(100, 1, vec![0, 3276, 0]), looping: false, gains: [1.0, 1.0] });
        let mut out = [0.0f32; 2 * 2];
        m.mix(&mut out, 2);
        assert!((out[2] - 0.05).abs() < 1e-4, "{out:?}");
    }

    #[test]
    fn a_loop_starts_over_and_pause_keeps_its_place() {
        let mut m = Mixer::new(100);
        m.apply(Command::Start { id: 7, sound: sound(100, 1, vec![100, 200, 300]), looping: true, gains: [1.0, 1.0] });
        let mut out = [0.0f32; 2 * 4];
        m.mix(&mut out, 2);
        let got: Vec<i32> = out.chunks(2).map(|f| (f[0] * 32_768.0).round() as i32).collect();
        assert_eq!(got, [100, 200, 300, 100]);
        m.apply(Command::Pause { id: 7 });
        m.mix(&mut out, 2);
        assert!(out.iter().all(|&x| x == 0.0));
        m.apply(Command::Resume { id: 7 });
        let mut one = [0.0f32; 2];
        m.mix(&mut one, 2);
        assert_eq!((one[0] * 32_768.0).round() as i32, 200, "goes on where it paused");
        m.apply(Command::Stop { id: 7 });
        assert_eq!(m.voices(), 0);
    }

    #[test]
    fn the_sum_is_clipped() {
        let mut m = Mixer::new(100);
        for id in 0..3 {
            m.apply(Command::Start { id, sound: sound(100, 1, vec![30_000]), looping: false, gains: [1.0, 1.0] });
        }
        let mut out = [0.0f32; 2];
        m.mix(&mut out, 2);
        assert_eq!(out, [1.0, 1.0]);
    }
}
