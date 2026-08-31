# M32 0.0.5 Audio Bundle B FIX3 — Pause Fade Final-frame Boundary

## Trigger

After FIX2, `m32-audio` compiled and Clippy passed.

The Bundle B audio runtime suite then reported:

```text
14 passed
1 failed
```

Failure:

```text
pause_fade_reaches_zero_in_exact_3840_frames_and_freezes_voice
```

The assertion correctly required the final fade frame, index 3839, to remain non-zero.

## Root Cause

`AudioRuntime::render_interleaved` previously performed:

```text
next_gain()
then is_fully_paused()
```

On the final fade frame, `next_gain()` changed:

```text
remaining 1 -> 0
gain        small positive -> target 0 internally
```

The following fully-paused check therefore suppressed the voice one frame too early even though the
gain returned for that output frame was still positive.

This was a real 80ms fade boundary off-by-one.

## FIX3

The runtime now captures the fully-paused state before advancing the envelope:

```text
was_fully_paused = is_fully_paused()
gain = next_gain()
render voice if !was_fully_paused
```

Resulting contract:

```text
frame 0       audible at gain 1.0
...
frame 3839    audible at gain > 0
frame 3840    silence, timeline frozen
```

Resume starts from the frozen voice position. Its first resumed frame is gain 0, followed by the
80ms fade-in.

## Scope

No changes to:

```text
48kHz / f32 stereo
60ms target
CPAL 0.18.2 API boundary
Wave normalization/resampling
mix saturation
guest MMAPI fixture
audio handles
repeat
MIDI note renderer
WIE adapter
dependencies
Cargo.lock dependency set
```

## Status

```text
0.0.5 Bundle B = FIX3 / IN_PROGRESS
```
