#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{thread, time::Duration};

    use m32_audio::{CpalOutputStream, OUTPUT_SAMPLE_RATE_HZ, RealtimeGuestAudioHost};
    use m32_emulator_api::{
        GuestAudioCommand, GuestAudioEventData, GuestAudioHost, GuestAudioSequence, GuestTimedAudioEvent,
    };

    let host = RealtimeGuestAudioHost::new();
    let stream = CpalOutputStream::open_default(&host)?;

    let frames = OUTPUT_SAMPLE_RATE_HZ as usize;
    let samples = (0..frames)
        .map(|frame| {
            let phase = std::f32::consts::TAU * 440.0 * frame as f32 / OUTPUT_SAMPLE_RATE_HZ as f32;
            (phase.sin() * 0.20 * f32::from(i16::MAX)) as i16
        })
        .collect::<Vec<_>>();

    host.dispatch(GuestAudioCommand::Play {
        handle: 0x4D32,
        sequence: GuestAudioSequence {
            duration: 1_000,
            events: vec![GuestTimedAudioEvent {
                time: 0,
                data: GuestAudioEventData::Wave {
                    channels: 1,
                    sampling_rate: OUTPUT_SAMPLE_RATE_HZ,
                    samples,
                },
            }],
        },
        repeat: false,
    })?;

    println!("M32 Windows audio smoke");
    println!("device: {}", stream.info().name);
    println!("sample rate: {} Hz", stream.info().sample_rate_hz);
    println!("channels: {}", stream.info().channels);
    match stream.info().requested_buffer_frames {
        Some(frames) => {
            println!("requested callback buffer: {frames} frames (60ms target)");
        }
        None => {
            println!("callback buffer: device default (2880-frame request unsupported/unknown)");
        }
    }
    println!("Playing a 440Hz M32 test tone for 1 second...");

    thread::sleep(Duration::from_millis(1_250));

    host.dispatch(GuestAudioCommand::Stop { handle: 0x4D32 })?;
    println!("M32 audio smoke finished.");
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    println!("This smoke probe is intended for Windows.");
}
