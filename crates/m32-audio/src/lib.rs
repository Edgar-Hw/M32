//! Deterministic M32 audio core.
//!
//! This crate owns backend-independent audio transformation and buffering policy.
//! OS/device output is intentionally introduced in a later Audio bundle.

use std::{collections::VecDeque, sync::Mutex};

use m32_emulator_api::{GuestAudioCommand, GuestAudioHost, GuestAudioHostError};

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
