# M32 0.0.5 Audio Bundle B FIX2 — CPAL 0.18.2 API Alignment

## Trigger

`cargo check -p m32-audio` failed after introducing the Windows CPAL boundary.

The six compile failures were all API-version mismatches against the pinned CPAL `0.18.2`.

## Root Cause

The initial T009 implementation used API forms from older CPAL releases.

Pinned CPAL `0.18.2` changed the relevant surfaces:

```text
Device::name()
-> DeviceTrait::description() -> DeviceDescription::name()

SampleRate wrapper
-> SampleRate is a u32 type alias

min_sample_rate().0 / max_sample_rate().0
-> min_sample_rate() / max_sample_rate()

with_sample_rate(SampleRate(x))
-> with_sample_rate(x)

build_output_stream(&config, ...)
-> build_output_stream(config, ...)

config.sample_rate.0
-> config.sample_rate
```

## FIX2 Scope

Only the CPAL 0.18.2 API call shapes are changed.

No change to:

```text
48000 Hz canonical rate
f32 stereo output requirement
2880-frame / 60ms target
3840-frame / 80ms pause fade
Wave decode/resampling
mixer behavior
voice/repeat/Stop behavior
baseline MIDI renderer
J2ME MMAPI fixture
WIE adapter
audio command contract
CPAL version pin
```

## Cargo.lock Note

CPAL is a cross-platform crate. Cargo.lock can record target-specific dependency packages for
multiple supported targets even though the M32 Windows build only compiles the dependencies selected
for `x86_64-pc-windows-msvc`.

## Status

```text
0.0.5 Bundle B = FIX2 / IN_PROGRESS
```
