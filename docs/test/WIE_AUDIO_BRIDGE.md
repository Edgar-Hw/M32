# M32 Guest Audio Host Contract and WIE AudioSink Bridge

Status: LOCKED BASELINE
Task: `0.0.2-T009`
Emulator API schema: `1`

## 1. 목적

T009 translates the pinned WIE `AudioSink` command stream into M32-owned audio command types.

This is a Core Adapter boundary Task.

It does not open an audio device, resample PCM, synthesize MIDI, implement latency buffering, or
produce audible output.

Those belong to the later `0.0.5 Audio Bridge` implementation.

## 2. Pinned WIE Audio Surface

At WIE revision:

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

WIE exposes:

```text
AudioHandle = u32

AudioCommand
- Play { handle, sequence: Arc<AudioSequence>, repeat }
- Stop { handle }

AudioSequence
- duration: u64
- events: Vec<TimedAudioEvent>

TimedAudioEvent
- time: u64
- data: AudioEventData

AudioEventData
- Midi(Vec<u8>)
- Wave {
    channels: u8,
    sampling_rate: u32,
    samples: Vec<i16>
  }

AudioSink: Send + Sync
- send(AudioCommand)
```

## 3. M32-owned Audio Types

`m32-emulator-api` owns:

```text
GuestAudioHandle = u32
GuestAudioCommand
GuestAudioSequence
GuestTimedAudioEvent
GuestAudioEventData
GuestAudioHost
GuestAudioHostError
```

No WIE audio type appears in the M32 public contract.

## 4. Timeline Unit Policy

Pinned WIE exposes `duration: u64` and event `time: u64`, but the type definition itself does not
encode a unit in the Rust type system.

T009 therefore preserves those values exactly as opaque WIE timeline values.

It does not rename them to milliseconds, samples, or ticks.

A later audio implementation may interpret them only after the corresponding WIE producer semantics
are verified.

## 5. MIDI Mapping

WIE:

```text
AudioEventData::Midi(Vec<u8>)
```

maps to:

```text
GuestAudioEventData::Midi(Vec<u8>)
```

Bytes are copied exactly.

The Core Adapter does not parse MIDI messages.

## 6. Wave Mapping

WIE Wave fields are preserved exactly:

```text
channels: u8
sampling_rate: u32
samples: Vec<i16>
```

T009 does not resample to M32's future 48 kHz output contract and does not force stereo.

That conversion belongs to `m32-audio`.

## 7. Play/Stop Mapping

Play preserves:

```text
handle
sequence duration
event times
event data
repeat
```

Stop preserves:

```text
handle
```

WIE `Arc<AudioSequence>` ownership does not escape the adapter. The M32 command owns its copied
sequence data.

## 8. Host Failure Semantics

Pinned `AudioSink::send()` has no return value.

M32 `GuestAudioHost::dispatch()` may return:

```text
GuestAudioHostErrorCode::DispatchFailed
```

If dispatch fails, the adapter:

- emits a structured `m32::audio` warning;
- logs only the stable error code;
- does not log MIDI bytes, PCM samples, or free-form host error text;
- does not panic;
- cannot return an error to WIE.

## 9. Non-Goals

T009 does not:

- create a Windows audio device;
- use cpal/rodio;
- resample to 48 kHz;
- convert mono/stereo layouts;
- synthesize MIDI/SMAF;
- implement the 40/60/80 ms latency modes;
- implement pause fade;
- assemble the complete WIE Platform.

## 10. Acceptance

- M32 audio types are WIE-independent;
- handle remains exact `u32`;
- MIDI bytes are preserved;
- Wave channels/rate/i16 samples are preserved;
- Play duration/time/repeat are preserved;
- Stop handle is preserved;
- host dispatch failure is non-panicking and sanitized in logs;
- all quality gates pass.
