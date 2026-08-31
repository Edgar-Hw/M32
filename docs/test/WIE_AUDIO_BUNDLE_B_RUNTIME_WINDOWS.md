# M32 0.0.5 Audio — Bundle B Functional Stage (T006-T009)

This is the second accelerated Audio bundle.

```text
T006 Deterministic J2ME MMAPI/SMAF Fixture
T007 Real Guest -> WIE -> M32 Audio Integration
T008 Sequence / Voice / Repeat / Baseline MIDI Renderer
T009 Windows CPAL Output + 60ms Target + 80ms Pause Fade
```

T010 version close is attached only after the T006-T009 evidence/manual device gate is complete.

## Pinned WIE MMAPI Boundary

Pinned WIE `Manager.createPlayer(InputStream, String)` supports
`application/vnd.smaf` and constructs `net.wie.SmafPlayer`.

The M32 fixture uses a deterministic empty SMAF stream. Pinned WIE accepts an empty SMAF parse as an
empty sequence, which is ideal for testing MMAPI transport without depending on copyrighted media.

The guest executes:

```text
Manager.createPlayer(..., "application/vnd.smaf")
Player.start()
Player.stop()
```

Expected M32 host commands:

```text
Play { handle=0, duration=0, events=[], repeat=false }
Stop { handle=0 }
```

Fixture SHA-256:

```text
JAD 1d8f5de025d5c201df5992e28070037343049b544ee6d93571030046c9f827d1
JAR ed88d129e90388aa045bebef5a4389e7751fa88890ad8f607b35761ba536dbc7
AudioMidlet.class b6f11e1bd44c438aa000f45dffd1baf0ef05708806e3afab70e34d3edf80c0c3
```

Classfile major: 52.

## Runtime Renderer

`RealtimeGuestAudioHost` implements the same stable `GuestAudioHost` that the WIE adapter consumes.

Wave events are canonicalized once on Play and scheduled by exact event millisecond -> 48k frame
position.

Voice identity uses the stable `u32` audio handle.

Stop removes that voice before subsequent rendering.

Repeat restarts the prepared sequence deterministically.

## Baseline MIDI

The first playable MIDI baseline implements:

```text
Note On
Note Off
Note On velocity=0 as Note Off
All Sound Off (CC120)
All Notes Off (CC123)
```

It renders deterministic sine voices at MIDI note pitch.

This deliberately does NOT claim General MIDI/SMAF instrument timbre fidelity yet. Program change,
pitch bend, SysEx timbre, and SoundFont-quality synthesis remain compatibility-expansion work.

Transport bytes are still preserved end-to-end.

## Pause

Pause fade remains exactly:

```text
80ms
3840 frames @ 48kHz
```

After fade-out reaches zero, voice timeline freezes.

Resume starts from the frozen position with an 80ms fade-in.

## Windows Output

Pinned dependency:

```text
cpal = 0.18.2
```

M32 requests an exact:

```text
f32
2 channels
48000 Hz
```

output stream from the Windows default device.

When the device exposes a callback buffer range that includes 2880 frames, M32 requests:

```text
2880 frames = 60ms
```

Otherwise M32 uses the device default buffer. The 60ms value is therefore a target, not a false
hardware guarantee.

The CPAL documentation explicitly notes that fixed callback size is a request and the host/hardware
may vary the actual callback size.

## Manual Device Gate

T009 cannot be fully proven in headless CI because a real speaker/device is physical state.

Run locally:

```powershell
cargo run -p m32-audio --example windows_audio_smoke
```

Acceptance requires:

```text
program exits successfully
default output device is reported
48000 Hz / 2 channels reported
a 440Hz tone is audibly heard for about one second
```

This manual result is recorded in T009 evidence before T010 close.

## Expected Test Baseline

Bundle A:

```text
m32-audio       10
m32-wie-adapter 66
workspace min   128
```

Bundle B functional additions:

```text
m32-audio       +5 => 15
m32-wie-adapter +2 => 68
workspace min         135
```
