# M32 0.0.5 Audio — T010 Canonical Integration / Version Close

Task:

```text
0.0.5-T010 Audio Canonical Integration / Version Close
```

T010 adds no new guest-visible audio feature. It closes the Audio version after the complete
T001-T009 implementation/evidence set.

## Closed Audio Contract

```text
48kHz canonical output
f32 stereo
i16 mono/stereo normalization
deterministic linear resampling
deterministic stereo mixing/saturation
thread-safe GuestAudioHost ingress
real J2ME MMAPI/SMAF guest transport
real WIE -> M32 audio bridge
exact timed Wave scheduling
u32 handle voice lifecycle
Stop
repeat
baseline MIDI NoteOn/NoteOff rendering
Windows CPAL output
60ms target latency
80ms / 3840-frame pause fade
physical audible Windows smoke
```

## Dependency Boundary

Core/unconditional:

```text
m32-audio -> m32-emulator-api
```

Windows-specific:

```text
cpal = =0.18.2
```

No direct WIE dependency is added to `m32-audio`.

## Expected Final Regression Baseline

```text
m32-desktop          9
m32-domain           3
m32-emulator-api    32
m32-input             6
m32-audio            15
m32-test-fixtures     2
m32-wie-adapter      68
-----------------------
minimum total       135
```

Zero-test crates are additional workspace members and do not change this minimum.

## Manual Audio Gate

T009 evidence must contain:

```text
MANUAL_AUDIBLE_SMOKE: PASS
```

This prevents T010 from closing only from a headless compile result.

## Non-claims

0.0.5 does not claim:

```text
full General MIDI instrument fidelity
SoundFont-quality synthesis
complete SMAF/vendor timbre fidelity
all J2ME media MIME types
per-game audio compatibility profiles
universal fixed 60ms hardware latency
```

Those remain later compatibility/product-quality scope.

## Next Version

```text
0.0.6 Storage / Persistence
10 tasks
```
