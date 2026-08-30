# M32 WIE Platform Assembly

Status: LOCKED BASELINE
Task: `0.0.2-T010`
WIE revision: `f0513eb758c02736981f545ad030eed937d55f3e`

## 1. 목적

T010 assembles every M32-owned host-service bridge created in T005 through T009 into one concrete
implementation of the pinned WIE `Platform` trait.

This is the final host-service composition step before emulator construction.

T010 does not load a game or instantiate a J2ME/SKT/LGT emulator.

## 2. Pinned WIE Platform Contract

The locked WIE `Platform: Send + Sync` requires:

```text
screen()
now()
database_repository()
filesystem()
audio_sink()
write_stdout()
write_stderr()
exit()
vibrate()
```

T010 implements all nine methods in:

```text
WiePlatformAdapter
```

## 3. Host Bundle

`m32-wie-adapter` introduces:

```text
WiePlatformHosts
```

with M32-owned service trait objects:

```text
DisplayHost
ClockHost
GuestDatabaseRepositoryHost
GuestFilesystemHost
GuestAudioHost
GuestOutputHost
ExitHost
VibrationHost
```

Stdout and stderr intentionally share one `GuestOutputHost`, so eight trait objects satisfy the
nine WIE service categories.

`WiePlatformHosts` contains no WIE type.

## 4. Assembly Mapping

```text
Platform::screen()
-> WieScreenAdapter
-> DisplayHost

Platform::now()
-> WieBasicHostBridge
-> ClockHost
-> Instant::from_epoch_millis(...)

Platform::database_repository()
-> WieDatabaseRepositoryAdapter
-> GuestDatabaseRepositoryHost

Platform::filesystem()
-> WieFilesystemAdapter
-> GuestFilesystemHost

Platform::audio_sink()
-> new WieAudioSinkAdapter
-> shared Arc<dyn GuestAudioHost>

Platform::write_stdout / write_stderr
-> WieBasicHostBridge
-> GuestOutputHost

Platform::exit()
-> WieBasicHostBridge
-> ExitHost::request_exit()

Platform::vibrate()
-> WieBasicHostBridge
-> VibrationHost
```

## 5. AudioSink Ownership

Pinned WIE requires:

```text
audio_sink() -> Box<dyn AudioSink>
```

Each call creates a fresh `WieAudioSinkAdapter`.

All sink adapters share the same:

```text
Arc<dyn GuestAudioHost>
```

This satisfies WIE ownership without duplicating the future M32 audio device/mixer host.

## 6. Emulator Construction Compatibility

Pinned J2ME construction accepts:

```text
J2MEEmulator::from_jar(
    platform: Box<dyn Platform>,
    jar_filename,
    jar_bytes
)
```

Therefore a `WiePlatformAdapter` can be boxed directly by the later launch/factory Task.

T010 intentionally does not expose a public method returning `Box<dyn wie_backend::Platform>`.
The later WIE-specific factory inside `m32-wie-adapter` owns that boxing step.

## 7. No Host Runtime Yet

The concrete services supplied to `WiePlatformHosts` are still abstract M32 host traits.

T010 does not:

- create the Windows display window;
- read the real wall clock;
- persist guest filesystem data;
- persist guest database records;
- open the audio device;
- terminate the desktop process;
- access controller rumble hardware.

Those concrete host implementations belong to their dedicated M32 crates/versions.

## 8. Non-Goals

T010 does not:

- add `wie_j2me`, `wie_skt`, `wie_lgt`, or other runtime crates;
- load JAR/JAD/game files;
- construct `WieSession`;
- drive emulator ticks;
- translate input events;
- present a real window frame.

## 9. Acceptance

- `WiePlatformAdapter` implements the pinned WIE `Platform`;
- all nine WIE service methods delegate to the correct M32 host;
- screen size and clock value survive the assembled platform;
- database name/app_id and filesystem AID/path survive assembly;
- stdout/stderr, exit, and vibration survive assembly;
- separate WIE audio sinks share one M32 audio host;
- no new dependency is introduced;
- existing M32/WIE dependency boundary remains valid;
- all quality gates pass.
