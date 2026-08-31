//! Deterministic M32 audio core.
//!
//! This crate owns backend-independent audio transformation and buffering policy.
//! Bundle B adds deterministic sequence rendering and the Windows CPAL output boundary.

use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    sync::{Arc, Mutex},
};

use m32_emulator_api::{
    GuestAudioCommand, GuestAudioEventData, GuestAudioHost, GuestAudioHostError, GuestAudioSequence,
};

pub const OUTPUT_SAMPLE_RATE_HZ: u32 = 48_000;
pub const OUTPUT_CHANNELS: u8 = 2;
pub const TARGET_LATENCY_MS: u32 = 60;
pub const PAUSE_FADE_MS: u32 = 80;

pub const TARGET_LATENCY_FRAMES: usize = OUTPUT_SAMPLE_RATE_HZ as usize * TARGET_LATENCY_MS as usize / 1_000;
pub const PAUSE_FADE_FRAMES: usize = OUTPUT_SAMPLE_RATE_HZ as usize * PAUSE_FADE_MS as usize / 1_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StereoFrame {
    pub left: f32,
    pub right: f32,
}

impl StereoFrame {
    pub const SILENCE: Self = Self { left: 0.0, right: 0.0 };

    #[must_use]
    pub const fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioTransformError {
    UnsupportedChannelCount(u8),
    MalformedInterleavedSamples { channels: u8, sample_count: usize },
    InvalidSamplingRate(u32),
    OutputLengthOverflow,
}

fn i16_to_f32(sample: i16) -> f32 {
    f32::from(sample) / 32_768.0
}

pub fn decode_i16_interleaved_to_stereo(
    channels: u8,
    samples: &[i16],
) -> Result<Vec<StereoFrame>, AudioTransformError> {
    match channels {
        1 => Ok(samples
            .iter()
            .copied()
            .map(|sample| {
                let value = i16_to_f32(sample);
                StereoFrame::new(value, value)
            })
            .collect()),
        2 => {
            if !samples.len().is_multiple_of(2) {
                return Err(AudioTransformError::MalformedInterleavedSamples {
                    channels,
                    sample_count: samples.len(),
                });
            }

            Ok(samples
                .as_chunks::<2>()
                .0
                .iter()
                .map(|frame| StereoFrame::new(i16_to_f32(frame[0]), i16_to_f32(frame[1])))
                .collect())
        }
        _ => Err(AudioTransformError::UnsupportedChannelCount(channels)),
    }
}

pub fn resample_stereo_to_output_rate(
    source_sampling_rate: u32,
    source: &[StereoFrame],
) -> Result<Vec<StereoFrame>, AudioTransformError> {
    if source_sampling_rate == 0 {
        return Err(AudioTransformError::InvalidSamplingRate(source_sampling_rate));
    }

    if source.is_empty() {
        return Ok(Vec::new());
    }

    if source_sampling_rate == OUTPUT_SAMPLE_RATE_HZ {
        return Ok(source.to_vec());
    }

    let numerator = (source.len() as u64)
        .checked_mul(u64::from(OUTPUT_SAMPLE_RATE_HZ))
        .ok_or(AudioTransformError::OutputLengthOverflow)?;

    let output_len_u64 = numerator.div_ceil(u64::from(source_sampling_rate));
    let output_len = usize::try_from(output_len_u64).map_err(|_| AudioTransformError::OutputLengthOverflow)?;

    let output_rate = u128::from(OUTPUT_SAMPLE_RATE_HZ);
    let source_rate = u128::from(source_sampling_rate);

    let mut output = Vec::with_capacity(output_len);

    for output_index in 0..output_len {
        let position_numerator = (output_index as u128).saturating_mul(source_rate);
        let base_index =
            usize::try_from(position_numerator / output_rate).map_err(|_| AudioTransformError::OutputLengthOverflow)?;

        if base_index >= source.len() - 1 {
            output.push(source[source.len() - 1]);
            continue;
        }

        let fraction = (position_numerator % output_rate) as f32 / OUTPUT_SAMPLE_RATE_HZ as f32;

        let first = source[base_index];
        let second = source[base_index + 1];

        output.push(StereoFrame::new(
            first.left + (second.left - first.left) * fraction,
            first.right + (second.right - first.right) * fraction,
        ));
    }

    Ok(output)
}

pub fn canonicalize_wave_to_output(
    channels: u8,
    sampling_rate: u32,
    samples: &[i16],
) -> Result<Vec<StereoFrame>, AudioTransformError> {
    let decoded = decode_i16_interleaved_to_stereo(channels, samples)?;
    resample_stereo_to_output_rate(sampling_rate, &decoded)
}

#[must_use]
pub fn mix_stereo_clips(clips: &[&[StereoFrame]], frame_count: usize) -> Vec<StereoFrame> {
    let mut output = vec![StereoFrame::SILENCE; frame_count];

    for clip in clips {
        for (target, source) in output.iter_mut().zip(clip.iter().copied()) {
            target.left += source.left;
            target.right += source.right;
        }
    }

    for frame in &mut output {
        frame.left = frame.left.clamp(-1.0, 1.0);
        frame.right = frame.right.clamp(-1.0, 1.0);
    }

    output
}

#[derive(Default)]
pub struct BufferedGuestAudioHost {
    commands: Mutex<VecDeque<GuestAudioCommand>>,
}

impl BufferedGuestAudioHost {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn drain(&self) -> Result<Vec<GuestAudioCommand>, GuestAudioHostError> {
        let mut commands = self
            .commands
            .lock()
            .map_err(|_| GuestAudioHostError::dispatch_failed("M32 audio command queue mutex poisoned"))?;

        Ok(commands.drain(..).collect())
    }

    pub fn len(&self) -> Result<usize, GuestAudioHostError> {
        self.commands
            .lock()
            .map(|commands| commands.len())
            .map_err(|_| GuestAudioHostError::dispatch_failed("M32 audio command queue mutex poisoned"))
    }

    pub fn is_empty(&self) -> Result<bool, GuestAudioHostError> {
        self.commands
            .lock()
            .map(|commands| commands.is_empty())
            .map_err(|_| GuestAudioHostError::dispatch_failed("M32 audio command queue mutex poisoned"))
    }
}

impl GuestAudioHost for BufferedGuestAudioHost {
    fn dispatch(&self, command: GuestAudioCommand) -> Result<(), GuestAudioHostError> {
        self.commands
            .lock()
            .map_err(|_| GuestAudioHostError::dispatch_failed("M32 audio command queue mutex poisoned"))?
            .push_back(command);

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioRuntimeError {
    Transform(AudioTransformError),
    MutexPoisoned,
}

impl fmt::Display for AudioRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transform(error) => write!(formatter, "audio transform failed: {error:?}"),
            Self::MutexPoisoned => write!(formatter, "audio runtime mutex poisoned"),
        }
    }
}

impl std::error::Error for AudioRuntimeError {}

impl From<AudioTransformError> for AudioRuntimeError {
    fn from(error: AudioTransformError) -> Self {
        Self::Transform(error)
    }
}

#[derive(Debug, Clone)]
struct PreparedWave {
    start_frame: usize,
    frames: Vec<StereoFrame>,
}

#[derive(Debug, Clone)]
struct PreparedMidi {
    frame: usize,
    order: usize,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct PreparedSequence {
    duration_frames: usize,
    waves: Vec<PreparedWave>,
    midi: Vec<PreparedMidi>,
}

impl PreparedSequence {
    fn from_guest(sequence: &GuestAudioSequence) -> Result<Self, AudioTransformError> {
        let mut waves = Vec::new();
        let mut midi = Vec::new();
        let mut max_frame = millis_to_frames(sequence.duration);

        for (order, event) in sequence.events.iter().enumerate() {
            let start_frame = millis_to_frames(event.time);
            match &event.data {
                GuestAudioEventData::Wave {
                    channels,
                    sampling_rate,
                    samples,
                } => {
                    let frames = canonicalize_wave_to_output(*channels, *sampling_rate, samples)?;
                    max_frame = max_frame.max(start_frame.saturating_add(frames.len()));
                    waves.push(PreparedWave { start_frame, frames });
                }
                GuestAudioEventData::Midi(bytes) => {
                    max_frame = max_frame.max(start_frame.saturating_add(1));
                    midi.push(PreparedMidi {
                        frame: start_frame,
                        order,
                        bytes: bytes.clone(),
                    });
                }
            }
        }

        midi.sort_by_key(|event| (event.frame, event.order));

        Ok(Self {
            duration_frames: max_frame.max(1),
            waves,
            midi,
        })
    }
}

fn millis_to_frames(milliseconds: u64) -> usize {
    let frames = milliseconds.saturating_mul(u64::from(OUTPUT_SAMPLE_RATE_HZ)) / 1_000;
    usize::try_from(frames).unwrap_or(usize::MAX)
}

#[derive(Debug, Clone, Copy)]
struct MidiNote {
    phase: f32,
    phase_step: f32,
    amplitude: f32,
}

#[derive(Debug, Clone)]
struct Voice {
    sequence: PreparedSequence,
    repeat: bool,
    position: usize,
    midi_cursor: usize,
    midi_notes: BTreeMap<(u8, u8), MidiNote>,
    finished: bool,
}

impl Voice {
    fn new(sequence: PreparedSequence, repeat: bool) -> Self {
        Self {
            sequence,
            repeat,
            position: 0,
            midi_cursor: 0,
            midi_notes: BTreeMap::new(),
            finished: false,
        }
    }

    fn render_frame(&mut self) -> StereoFrame {
        if self.finished {
            return StereoFrame::SILENCE;
        }

        self.apply_midi_events_at_current_frame();

        let mut frame = StereoFrame::SILENCE;
        for wave in &self.sequence.waves {
            if self.position < wave.start_frame {
                continue;
            }

            let local = self.position - wave.start_frame;
            if let Some(sample) = wave.frames.get(local) {
                frame.left += sample.left;
                frame.right += sample.right;
            }
        }

        let midi_sample = self.render_midi_sample();
        frame.left += midi_sample;
        frame.right += midi_sample;
        frame.left = frame.left.clamp(-1.0, 1.0);
        frame.right = frame.right.clamp(-1.0, 1.0);

        self.position = self.position.saturating_add(1);
        if self.position >= self.sequence.duration_frames {
            if self.repeat {
                self.position = 0;
                self.midi_cursor = 0;
                self.midi_notes.clear();
            } else {
                self.finished = true;
            }
        }

        frame
    }

    fn apply_midi_events_at_current_frame(&mut self) {
        while let Some(event) = self.sequence.midi.get(self.midi_cursor) {
            if event.frame != self.position {
                break;
            }

            apply_midi_message(&mut self.midi_notes, &event.bytes);
            self.midi_cursor += 1;
        }
    }

    fn render_midi_sample(&mut self) -> f32 {
        let mut sample = 0.0_f32;
        for note in self.midi_notes.values_mut() {
            sample += note.phase.sin() * note.amplitude;
            note.phase += note.phase_step;
            if note.phase >= std::f32::consts::TAU {
                note.phase -= std::f32::consts::TAU;
            }
        }
        sample.clamp(-1.0, 1.0)
    }
}

fn apply_midi_message(notes: &mut BTreeMap<(u8, u8), MidiNote>, bytes: &[u8]) {
    let Some(&status) = bytes.first() else {
        return;
    };

    let message = status & 0xF0;
    let channel = status & 0x0F;

    match message {
        0x80 if bytes.len() >= 3 => {
            notes.remove(&(channel, bytes[1]));
        }
        0x90 if bytes.len() >= 3 => {
            let note = bytes[1];
            let velocity = bytes[2];
            if velocity == 0 {
                notes.remove(&(channel, note));
                return;
            }

            let semitones = (f32::from(note) - 69.0) / 12.0;
            let frequency_hz = 440.0 * 2.0_f32.powf(semitones);
            let phase_step = std::f32::consts::TAU * frequency_hz / OUTPUT_SAMPLE_RATE_HZ as f32;
            let amplitude = f32::from(velocity) / 127.0 * 0.15;
            notes.insert(
                (channel, note),
                MidiNote {
                    phase: 0.0,
                    phase_step,
                    amplitude,
                },
            );
        }
        0xB0 if bytes.len() >= 3 && matches!(bytes[1], 120 | 123) => {
            notes.retain(|(note_channel, _), _| *note_channel != channel);
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Copy)]
struct PauseEnvelope {
    paused: bool,
    gain: f32,
    target: f32,
    remaining: usize,
}

impl Default for PauseEnvelope {
    fn default() -> Self {
        Self {
            paused: false,
            gain: 1.0,
            target: 1.0,
            remaining: 0,
        }
    }
}

impl PauseEnvelope {
    fn set_paused(&mut self, paused: bool) {
        if self.paused == paused && self.remaining == 0 {
            return;
        }

        self.paused = paused;
        self.target = if paused { 0.0 } else { 1.0 };
        self.remaining = PAUSE_FADE_FRAMES;
    }

    fn next_gain(&mut self) -> f32 {
        let gain = self.gain;
        if self.remaining > 0 {
            let step = (self.target - self.gain) / self.remaining as f32;
            self.gain += step;
            self.remaining -= 1;
            if self.remaining == 0 {
                self.gain = self.target;
            }
        }
        gain
    }

    fn is_fully_paused(&self) -> bool {
        self.paused && self.remaining == 0 && self.gain == 0.0
    }
}

#[derive(Debug, Default)]
struct AudioRuntime {
    voices: BTreeMap<u32, Voice>,
    pause: PauseEnvelope,
}

impl AudioRuntime {
    fn handle_command(&mut self, command: GuestAudioCommand) -> Result<(), AudioRuntimeError> {
        match command {
            GuestAudioCommand::Play {
                handle,
                sequence,
                repeat,
            } => {
                let prepared = PreparedSequence::from_guest(&sequence)?;
                self.voices.insert(handle, Voice::new(prepared, repeat));
            }
            GuestAudioCommand::Stop { handle } => {
                self.voices.remove(&handle);
            }
        }

        Ok(())
    }

    fn render_interleaved(&mut self, output: &mut [f32]) {
        if !output.len().is_multiple_of(OUTPUT_CHANNELS as usize) {
            output.fill(0.0);
            return;
        }

        for samples in output.as_chunks_mut::<2>().0 {
            let was_fully_paused = self.pause.is_fully_paused();
            let gain = self.pause.next_gain();
            let mut mixed = StereoFrame::SILENCE;

            if !was_fully_paused {
                for voice in self.voices.values_mut() {
                    let frame = voice.render_frame();
                    mixed.left += frame.left;
                    mixed.right += frame.right;
                }
            }

            self.voices.retain(|_, voice| !voice.finished);

            samples[0] = (mixed.left * gain).clamp(-1.0, 1.0);
            samples[1] = (mixed.right * gain).clamp(-1.0, 1.0);
        }
    }
}

#[derive(Clone, Default)]
pub struct RealtimeGuestAudioHost {
    runtime: Arc<Mutex<AudioRuntime>>,
}

impl RealtimeGuestAudioHost {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_paused(&self, paused: bool) -> Result<(), AudioRuntimeError> {
        self.runtime
            .lock()
            .map_err(|_| AudioRuntimeError::MutexPoisoned)?
            .pause
            .set_paused(paused);
        Ok(())
    }

    pub fn render_for_test(&self, frame_count: usize) -> Result<Vec<StereoFrame>, AudioRuntimeError> {
        let mut interleaved = vec![0.0_f32; frame_count.saturating_mul(2)];
        self.runtime
            .lock()
            .map_err(|_| AudioRuntimeError::MutexPoisoned)?
            .render_interleaved(&mut interleaved);

        Ok(interleaved
            .as_chunks::<2>()
            .0
            .iter()
            .map(|frame| StereoFrame::new(frame[0], frame[1]))
            .collect())
    }

    #[cfg(windows)]
    fn runtime(&self) -> Arc<Mutex<AudioRuntime>> {
        self.runtime.clone()
    }
}

impl GuestAudioHost for RealtimeGuestAudioHost {
    fn dispatch(&self, command: GuestAudioCommand) -> Result<(), GuestAudioHostError> {
        self.runtime
            .lock()
            .map_err(|_| GuestAudioHostError::dispatch_failed("M32 realtime audio mutex poisoned"))?
            .handle_command(command)
            .map_err(|error| GuestAudioHostError::dispatch_failed(error.to_string()))
    }
}

#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputDeviceInfo {
    pub name: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub requested_buffer_frames: Option<u32>,
}

#[cfg(windows)]
#[derive(Debug)]
pub enum OutputStreamError {
    NoDefaultDevice,
    QueryConfigs(String),
    NoCanonicalF32Stereo48kConfig,
    BuildStream(String),
    PlayStream(String),
}

#[cfg(windows)]
impl fmt::Display for OutputStreamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDefaultDevice => write!(formatter, "no default Windows audio output device"),
            Self::QueryConfigs(error) => {
                write!(formatter, "failed to query output configs: {error}")
            }
            Self::NoCanonicalF32Stereo48kConfig => {
                write!(formatter, "default device has no f32 stereo 48kHz output config")
            }
            Self::BuildStream(error) => {
                write!(formatter, "failed to build audio stream: {error}")
            }
            Self::PlayStream(error) => {
                write!(formatter, "failed to start audio stream: {error}")
            }
        }
    }
}

#[cfg(windows)]
impl std::error::Error for OutputStreamError {}

#[cfg(windows)]
pub struct CpalOutputStream {
    _stream: cpal::Stream,
    info: OutputDeviceInfo,
}

#[cfg(windows)]
impl CpalOutputStream {
    pub fn open_default(host: &RealtimeGuestAudioHost) -> Result<Self, OutputStreamError> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use cpal::{BufferSize, SampleFormat, SupportedBufferSize};

        let cpal_host = cpal::default_host();
        let device = cpal_host
            .default_output_device()
            .ok_or(OutputStreamError::NoDefaultDevice)?;
        let name = device
            .description()
            .map(|description| description.name().to_owned())
            .unwrap_or_else(|_| "Unknown output device".to_owned());

        let mut selected = None;
        for range in device
            .supported_output_configs()
            .map_err(|error| OutputStreamError::QueryConfigs(error.to_string()))?
        {
            if range.channels() != u16::from(OUTPUT_CHANNELS)
                || range.sample_format() != SampleFormat::F32
                || range.min_sample_rate() > OUTPUT_SAMPLE_RATE_HZ
                || range.max_sample_rate() < OUTPUT_SAMPLE_RATE_HZ
            {
                continue;
            }

            selected = Some(range.with_sample_rate(OUTPUT_SAMPLE_RATE_HZ));
            break;
        }

        let supported = selected.ok_or(OutputStreamError::NoCanonicalF32Stereo48kConfig)?;

        let requested_buffer_frames = match supported.buffer_size() {
            SupportedBufferSize::Range { min, max }
                if *min <= TARGET_LATENCY_FRAMES as u32 && TARGET_LATENCY_FRAMES as u32 <= *max =>
            {
                Some(TARGET_LATENCY_FRAMES as u32)
            }
            _ => None,
        };

        let mut config = supported.config();
        config.buffer_size = requested_buffer_frames.map_or(BufferSize::Default, BufferSize::Fixed);

        let runtime = host.runtime();
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _| {
                    if let Ok(mut runtime) = runtime.try_lock() {
                        runtime.render_interleaved(data);
                    } else {
                        data.fill(0.0);
                    }
                },
                |error| eprintln!("M32 audio output stream error: {error}"),
                None,
            )
            .map_err(|error| OutputStreamError::BuildStream(error.to_string()))?;

        stream
            .play()
            .map_err(|error| OutputStreamError::PlayStream(error.to_string()))?;

        Ok(Self {
            _stream: stream,
            info: OutputDeviceInfo {
                name,
                sample_rate_hz: config.sample_rate,
                channels: config.channels,
                requested_buffer_frames,
            },
        })
    }

    #[must_use]
    pub fn info(&self) -> &OutputDeviceInfo {
        &self.info
    }
}

#[cfg(test)]
mod bundle_b_tests {
    use m32_emulator_api::{GuestAudioEventData, GuestAudioSequence, GuestTimedAudioEvent};

    use super::*;

    fn wave_play(handle: u32, repeat: bool, time: u64, samples: Vec<i16>) -> GuestAudioCommand {
        GuestAudioCommand::Play {
            handle,
            sequence: GuestAudioSequence {
                duration: 0,
                events: vec![GuestTimedAudioEvent {
                    time,
                    data: GuestAudioEventData::Wave {
                        channels: 1,
                        sampling_rate: OUTPUT_SAMPLE_RATE_HZ,
                        samples,
                    },
                }],
            },
            repeat,
        }
    }

    #[test]
    fn realtime_host_schedules_wave_at_exact_millisecond_frame() {
        let host = RealtimeGuestAudioHost::new();
        host.dispatch(wave_play(1, false, 10, vec![16_384, 16_384]))
            .expect("wave Play must be accepted");

        let rendered = host.render_for_test(483).expect("render must succeed");
        assert!(rendered[..480].iter().all(|frame| *frame == StereoFrame::SILENCE));
        assert_eq!(rendered[480], StereoFrame::new(0.5, 0.5));
        assert_eq!(rendered[481], StereoFrame::new(0.5, 0.5));
        assert_eq!(rendered[482], StereoFrame::SILENCE);
    }

    #[test]
    fn stop_by_handle_removes_active_voice_before_next_render() {
        let host = RealtimeGuestAudioHost::new();
        host.dispatch(wave_play(7, true, 0, vec![16_384, 16_384]))
            .expect("repeating Play must be accepted");

        assert_eq!(host.render_for_test(1).unwrap()[0], StereoFrame::new(0.5, 0.5));

        host.dispatch(GuestAudioCommand::Stop { handle: 7 })
            .expect("Stop must be accepted");

        assert!(
            host.render_for_test(8)
                .unwrap()
                .iter()
                .all(|frame| *frame == StereoFrame::SILENCE)
        );
    }

    #[test]
    fn repeat_restarts_prepared_wave_deterministically() {
        let host = RealtimeGuestAudioHost::new();
        host.dispatch(wave_play(3, true, 0, vec![8_192, -8_192]))
            .expect("repeat Play must be accepted");

        let rendered = host.render_for_test(6).unwrap();
        assert_eq!(rendered[0], StereoFrame::new(0.25, 0.25));
        assert_eq!(rendered[1], StereoFrame::new(-0.25, -0.25));
        assert_eq!(rendered[2], StereoFrame::new(0.25, 0.25));
        assert_eq!(rendered[3], StereoFrame::new(-0.25, -0.25));
    }

    #[test]
    fn baseline_midi_note_on_off_renders_signal_then_stops() {
        let host = RealtimeGuestAudioHost::new();
        host.dispatch(GuestAudioCommand::Play {
            handle: 4,
            sequence: GuestAudioSequence {
                duration: 20,
                events: vec![
                    m32_emulator_api::GuestTimedAudioEvent {
                        time: 0,
                        data: GuestAudioEventData::Midi(vec![0x90, 69, 127]),
                    },
                    m32_emulator_api::GuestTimedAudioEvent {
                        time: 10,
                        data: GuestAudioEventData::Midi(vec![0x80, 69, 0]),
                    },
                ],
            },
            repeat: false,
        })
        .expect("MIDI Play must be accepted");

        let rendered = host.render_for_test(960).unwrap();
        assert!(rendered[1..480].iter().any(|frame| frame.left.abs() > 0.001));
        assert!(rendered[480..].iter().all(|frame| frame.left.abs() < 0.000_001));
    }

    #[test]
    fn pause_fade_reaches_zero_in_exact_3840_frames_and_freezes_voice() {
        let host = RealtimeGuestAudioHost::new();
        host.dispatch(wave_play(5, true, 0, vec![16_384]))
            .expect("repeat Play must be accepted");
        host.set_paused(true).expect("pause must be accepted");

        let faded = host.render_for_test(PAUSE_FADE_FRAMES + 2).unwrap();

        assert_eq!(faded[0], StereoFrame::new(0.5, 0.5));
        assert!(faded[PAUSE_FADE_FRAMES - 1].left > 0.0);
        assert_eq!(faded[PAUSE_FADE_FRAMES], StereoFrame::SILENCE);
        assert_eq!(faded[PAUSE_FADE_FRAMES + 1], StereoFrame::SILENCE);

        host.set_paused(false).expect("resume must be accepted");
        let resumed = host.render_for_test(2).unwrap();
        assert_eq!(resumed[0], StereoFrame::SILENCE);
        assert!(resumed[1].left > 0.0);
    }
}

#[cfg(test)]
mod tests {
    use m32_emulator_api::{GuestAudioEventData, GuestAudioSequence, GuestTimedAudioEvent};

    use super::*;

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.000_01,
            "actual={actual}, expected={expected}"
        );
    }

    #[test]
    fn canonical_output_contract_is_48khz_f32_stereo() {
        assert_eq!(OUTPUT_SAMPLE_RATE_HZ, 48_000);
        assert_eq!(OUTPUT_CHANNELS, 2);
        assert_eq!(std::mem::size_of::<f32>(), 4);
        assert_eq!(std::mem::size_of::<StereoFrame>(), 8);
    }

    #[test]
    fn latency_and_pause_fade_contracts_have_exact_frame_counts() {
        assert_eq!(TARGET_LATENCY_MS, 60);
        assert_eq!(TARGET_LATENCY_FRAMES, 2_880);

        assert_eq!(PAUSE_FADE_MS, 80);
        assert_eq!(PAUSE_FADE_FRAMES, 3_840);
    }

    #[test]
    fn mono_i16_is_normalized_and_duplicated_to_stereo() {
        let decoded = decode_i16_interleaved_to_stereo(1, &[i16::MIN, 0, 16_384]).expect("mono PCM must decode");

        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0], StereoFrame::new(-1.0, -1.0));
        assert_eq!(decoded[1], StereoFrame::SILENCE);
        assert_near(decoded[2].left, 0.5);
        assert_near(decoded[2].right, 0.5);
    }

    #[test]
    fn stereo_i16_preserves_channel_order_and_rejects_malformed_input() {
        let decoded =
            decode_i16_interleaved_to_stereo(2, &[16_384, -16_384, 0, 8_192]).expect("stereo PCM must decode");

        assert_eq!(decoded.len(), 2);
        assert_near(decoded[0].left, 0.5);
        assert_near(decoded[0].right, -0.5);
        assert_eq!(decoded[1].left, 0.0);
        assert_near(decoded[1].right, 0.25);

        assert_eq!(
            decode_i16_interleaved_to_stereo(2, &[1, 2, 3]),
            Err(AudioTransformError::MalformedInterleavedSamples {
                channels: 2,
                sample_count: 3,
            })
        );

        assert_eq!(
            decode_i16_interleaved_to_stereo(3, &[]),
            Err(AudioTransformError::UnsupportedChannelCount(3))
        );
    }

    #[test]
    fn output_rate_input_round_trips_without_resampling() {
        let source = [StereoFrame::new(-0.25, 0.25), StereoFrame::new(0.5, -0.5)];

        assert_eq!(
            resample_stereo_to_output_rate(48_000, &source).expect("48k source must pass"),
            source
        );
    }

    #[test]
    fn linear_resampler_upsamples_24khz_to_48khz_deterministically() {
        let source = [StereoFrame::new(0.0, 0.0), StereoFrame::new(1.0, -1.0)];

        let output = resample_stereo_to_output_rate(24_000, &source).expect("24k source must resample");

        assert_eq!(output.len(), 4);
        assert_eq!(output[0], StereoFrame::new(0.0, 0.0));
        assert_near(output[1].left, 0.5);
        assert_near(output[1].right, -0.5);
        assert_eq!(output[2], StereoFrame::new(1.0, -1.0));
        assert_eq!(output[3], StereoFrame::new(1.0, -1.0));
    }

    #[test]
    fn resampler_rejects_zero_sampling_rate() {
        assert_eq!(
            resample_stereo_to_output_rate(0, &[StereoFrame::SILENCE]),
            Err(AudioTransformError::InvalidSamplingRate(0))
        );
    }

    #[test]
    fn canonical_wave_path_combines_decode_and_resample() {
        let output = canonicalize_wave_to_output(1, 24_000, &[0, 16_384]).expect("canonical wave path must succeed");

        assert_eq!(output.len(), 4);
        assert_eq!(output[0], StereoFrame::SILENCE);
        assert_near(output[1].left, 0.25);
        assert_near(output[1].right, 0.25);
        assert_near(output[2].left, 0.5);
        assert_near(output[3].left, 0.5);
    }

    #[test]
    fn deterministic_mixer_sums_and_saturates_each_stereo_channel() {
        let first = [StereoFrame::new(0.75, -0.75), StereoFrame::new(0.25, 0.25)];
        let second = [StereoFrame::new(0.75, -0.75), StereoFrame::new(-0.5, 0.5)];

        let mixed = mix_stereo_clips(&[&first, &second], 3);

        assert_eq!(mixed.len(), 3);
        assert_eq!(mixed[0], StereoFrame::new(1.0, -1.0));
        assert_eq!(mixed[1], StereoFrame::new(-0.25, 0.75));
        assert_eq!(mixed[2], StereoFrame::SILENCE);
    }

    #[test]
    fn buffered_guest_audio_host_preserves_play_stop_and_payload_order() {
        let host = BufferedGuestAudioHost::new();

        let play = GuestAudioCommand::Play {
            handle: 17,
            sequence: GuestAudioSequence {
                duration: 250,
                events: vec![
                    GuestTimedAudioEvent {
                        time: 0,
                        data: GuestAudioEventData::Midi(vec![0x90, 60, 100]),
                    },
                    GuestTimedAudioEvent {
                        time: 10,
                        data: GuestAudioEventData::Wave {
                            channels: 1,
                            sampling_rate: 8_000,
                            samples: vec![-1, 0, 1],
                        },
                    },
                ],
            },
            repeat: true,
        };
        let stop = GuestAudioCommand::Stop { handle: 17 };

        host.dispatch(play.clone()).expect("Play must enqueue");
        host.dispatch(stop.clone()).expect("Stop must enqueue");

        assert_eq!(host.len().expect("queue length"), 2);
        assert!(!host.is_empty().expect("queue emptiness"));
        assert_eq!(host.drain().expect("queue drain"), vec![play, stop]);
        assert!(host.is_empty().expect("queue emptiness after drain"));
    }
}
