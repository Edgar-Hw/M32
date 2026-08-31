# M32 0.0.5 Audio — Accelerated Bundle A (T001-T005)

Status: IMPLEMENTATION BASELINE

The Audio version is intentionally executed in two large bundles to shorten the path to First
Playable while preserving individual Task contracts, tests, evidence, and version-close gates.

```text
Bundle A: T001-T005
Bundle B: T006-T010
```

## Bundle A

```text
0.0.5-T001 Canonical Audio Output Contract
0.0.5-T002 PCM i16 Mono/Stereo Normalization
0.0.5-T003 Deterministic 48kHz Linear Resampler
0.0.5-T004 Deterministic Stereo Mix/Saturation
0.0.5-T005 Thread-safe GuestAudioHost Command Ingress
```

## T001

Locks:

```text
sample rate     = 48000 Hz
internal sample = f32
channels        = 2 stereo
target latency  = 60 ms = 2880 frames
pause fade      = 80 ms = 3840 frames
```

T001 only locks the contract. Actual device latency and pause fade execution are closed in Bundle B.

## T002

WIE Wave PCM enters M32 as interleaved `i16`.

Supported canonical decode:

```text
1 channel -> duplicate mono into L/R
2 channels -> preserve interleaved L/R order
```

Normalization:

```text
f32 = i16 / 32768.0
```

Malformed stereo input and unsupported channel counts are rejected explicitly.

## T003

All Wave PCM is converted to 48kHz before mixing.

The first implementation uses deterministic linear interpolation.

Output frame count:

```text
ceil(source_frames * 48000 / source_rate)
```

Sampling positions are derived from integer source/output-rate ratios.

No wall-clock or device-dependent state participates in resampling.

## T004

The deterministic mixer:

```text
sums active canonical stereo clips
clamps each channel to [-1.0, 1.0]
fills uncovered output with silence
```

Voice timeline and sequence scheduling are Bundle B scope.

## T005

`BufferedGuestAudioHost` implements the stable `GuestAudioHost`.

It provides a thread-safe FIFO ingress from the already-existing WIE audio bridge into `m32-audio`.

It preserves:

```text
Play handle
Stop handle
repeat flag
sequence duration
timed-event order
raw MIDI bytes
Wave channels/rate/i16 samples
```

No audio command is interpreted or synthesized at the WIE-specific adapter boundary.

## Dependency Boundary

The only new production edge is:

```text
m32-audio -> m32-emulator-api
```

No external dependency is added in Bundle A.

A Cargo.lock workspace metadata update is expected.

## Expected Test Counts

Input baseline:

```text
workspace minimum = 118
```

Bundle A adds:

```text
m32-audio = 10 tests
```

Expected minimum workspace total:

```text
128 tests
```

## Bundle B Preview

Bundle B closes the user-visible path:

```text
T006 deterministic J2ME audio fixture
T007 real guest -> WIE -> M32 audio integration
T008 sequence / voice / repeat / handle lifecycle
T009 Windows output, 60ms target, 80ms pause fade, safe silence boundary
T010 canonical verifier / Audio version close
```

Bundle B is not accepted until sound reaches a real Windows output device through the M32-owned
audio path.
