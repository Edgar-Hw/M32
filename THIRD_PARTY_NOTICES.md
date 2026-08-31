# M32 Third-Party Notices

Last reviewed for Task: `0.1.0-T004` (First Playable Bundle A)

This file records third-party software that M32 uses, bundles, or has reserved
as an explicitly planned upstream dependency in the locked implementation plan.

The presence of a project in a "planned" section does **not** mean that its
code is already linked into the current M32 build.

## 1. Current application build

### tracing

- Project: tracing
- Version: 0.1.41
- Upstream: https://github.com/tokio-rs/tracing
- Purpose in M32: structured application diagnostics and events
- License: MIT
- Copyright: Copyright (c) 2019 Tokio Contributors
- Current status: linked by `m32-desktop`
- Compatibility note: pinned to 0.1.41 so the M32 workspace can coexist with the
  pinned WIE `wie_util` requirement `tracing-attributes <0.1.29`

### tracing-subscriber

- Project: tracing-subscriber
- Version: 0.3.23
- Upstream: https://github.com/tokio-rs/tracing
- Purpose in M32: console subscriber/formatter for tracing events
- License: MIT
- Copyright: Copyright (c) 2019 Tokio Contributors
- Current status: linked by `m32-desktop`
- M32 feature selection: default features disabled; `fmt`, `std` enabled

### egui family

- Projects: egui, egui-winit, egui-wgpu
- Version: 0.36.1
- Upstream: https://github.com/emilk/egui
- Purpose in M32: immediate-mode UI, winit event integration, wgpu presentation
- License: MIT OR Apache-2.0
- Current status: linked by First Playable Bundle A
- M32 feature note: egui default fonts are enabled for the Bundle A shell
- Default font assets reported by upstream:
  - `emoji-icon-font.ttf`: MIT
  - `Hack-Regular.ttf`: MIT
  - `NotoEmoji-Regular.ttf`: SIL Open Font License
  - `Ubuntu-Light.ttf`: Ubuntu Font Licence
- M32 does not redistribute these font files as standalone user artifacts.
  Release packaging must preserve all applicable font notices/licenses.

### winit

- Project: winit
- Version: 0.30.13
- Upstream: https://github.com/rust-windowing/winit
- Purpose in M32: native Windows event loop and window creation
- License: Apache-2.0
- Current status: linked by `m32-desktop` and `m32-display`

### wgpu

- Project: wgpu
- Version: 30.0.1
- Upstream: https://github.com/gfx-rs/wgpu
- Purpose in M32: native GPU surface/device/queue rendering
- License: MIT OR Apache-2.0
- Current status: linked by `m32-display`

### pollster

- Project: pollster
- Version: 0.4.0
- Upstream: https://github.com/zesterer/pollster
- Purpose in M32: synchronous bootstrap of the async wgpu initialization boundary
- License: Apache-2.0/MIT
- Current status: linked by `m32-display`

### CPAL

- Project: CPAL
- Version: 0.18.2
- Upstream: https://github.com/RustAudio/cpal
- Purpose in M32: Windows audio output
- License: Apache-2.0
- Current status: linked by `m32-audio` on Windows

### rusqlite / bundled SQLite

- Project: rusqlite
- Version: 0.37.0
- Upstream: https://github.com/rusqlite/rusqlite
- Purpose in M32: persistent RMS/filesystem metadata and storage
- License: MIT
- Current status: linked by `m32-storage`
- M32 feature selection: `bundled`
- Bundled SQLite licensing note: SQLite is public domain according to the
  rusqlite upstream licensing documentation.

The Rust compiler, Cargo, rustfmt, Clippy, Windows SDK/MSVC tooling, Git, and
GitHub Actions runner tooling are development/build infrastructure and are not
bundled by M32 as application runtime content.

Whenever a new runtime/library/font/audio component is added, this notice must
be updated in the same Task or dependency-change commit.

## 2. Locked emulator upstream notices

### WIE

- Project: WIE
- Upstream: https://github.com/dlunch/wie
- Purpose in M32: WIPI / SKVM / J2ME emulation core behind `m32-wie-adapter`
- M32 baseline revision:
  `f0513eb758c02736981f545ad030eed937d55f3e`
- License: MIT
- Copyright: Copyright 2020 Inseok Lee
- Current status: linked by `m32-wie-adapter`; desktop does not directly depend
  on WIE in Bundle A

### SMAF / smaf_player

- Project: SMAF
- Upstream: https://github.com/dlunch/smaf
- Revision: `8009d78512fd121609a841f31aa527bf2a4af456`
- Components used: `smaf`, `smaf_player`
- Purpose in M32: transitive SMAF parsing/playback dependency required by the
  pinned WIE backend
- License: MIT
- Copyright: Copyright 2020 Inseok Lee
- Distribution form: minimal vendored source under `third_party/smaf`
- Current status: linked transitively through `wie_backend`

### RustJava

- Project: RustJava
- Upstream: https://github.com/dlunch/RustJava
- Revision: `ba5797b8eb4cf376fdd63129903d319d1d7acf98`
- Purpose in M32: JVM / Java runtime dependency consumed transitively by WIE
- License: MIT
- Copyright: Copyright 2020 Inseok Lee
- Distribution form: compatibility-patched vendored source under
  `third_party/rustjava`
- Current status: linked transitively by the WIE/J2ME integration

## 3. Components not yet selected

The following categories still deliberately have no final distributable
component selected in Bundle A:

- M32 final UI fonts (Noto Sans KR / Galmuri packaging work remains later)
- MIDI SoundFont
- image/audio preview codecs
- final installer/runtime packaging dependencies

A component must not be added to an M32 distributable until its source,
version/revision, license, copyright notice, redistribution obligations, and
distribution location have been reviewed and recorded here.

## 4. Release packaging rule

Every public M32 distributable must contain at minimum:

- `LICENSE`
- `THIRD_PARTY_NOTICES.md`
- the user-facing game-file policy, either as
  `docs/legal/GAME_FILE_POLICY.md` or an equivalent packaged copy

Before a release is declared complete, the actual shipped dependency/artifact
inventory must be compared against this file. A notice for an actually shipped
third-party component may not be omitted.

This document is an engineering compliance record, not legal advice.
