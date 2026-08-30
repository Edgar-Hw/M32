# M32 Third-Party Notices

Last reviewed for Task: `0.0.2-T001`

This file records third-party software that M32 uses, bundles, or has reserved
as an explicitly planned upstream dependency in the locked implementation plan.

The presence of a project in the "planned" section does **not** mean that its
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

The upstream MIT permission and warranty terms must be preserved in
distributions that contain substantial portions of these projects.

The Rust compiler, Cargo, rustfmt, Clippy, Windows SDK/MSVC tooling, Git, and
GitHub Actions runner tooling are development/build infrastructure and are not
bundled by M32 as application runtime content.

Whenever a new runtime/library/font/audio component is added, this notice must
be updated in the same Task or dependency-change commit.

## 2. Reserved upstream notices for the locked emulator plan

The following projects are listed now because the M32 locked architecture
contains components whose integration status is recorded individually below.

### WIE

- Project: WIE
- Upstream: https://github.com/dlunch/wie
- Purpose in M32: WIPI / SKVM / J2ME emulation core behind `m32-wie-adapter`
- Planned M32 baseline revision:
  `f0513eb758c02736981f545ad030eed937d55f3e`
- License: MIT
- Copyright: Copyright 2020 Inseok Lee
- Current 0.0.2-T001 status: `wie_backend` is a pinned compile dependency of `m32-wie-adapter`; WIE is not yet wired into the `m32-desktop` runtime

MIT permission and warranty terms are provided by the upstream project's
LICENSE and must be preserved in M32 distributions that include substantial
portions of WIE.


### SMAF / smaf_player

- Project: SMAF
- Upstream: https://github.com/dlunch/smaf
- Revision: `8009d78512fd121609a841f31aa527bf2a4af456`
- Components used: `smaf`, `smaf_player`
- Purpose in M32: transitive SMAF parsing/playback dependency required by the pinned WIE backend
- License: MIT
- Copyright: Copyright 2020 Inseok Lee
- Distribution form: minimal vendored source under `third_party/smaf`
- Current 0.0.2-T001 status: linked transitively through `wie_backend`

This revision is the SMAF revision recorded by the pinned WIE `Cargo.lock`. M32 vendors only the
library package sources required for the WIE backend and preserves the upstream MIT license.

### RustJava

- Project: RustJava
- Upstream: https://github.com/dlunch/RustJava
- Purpose in M32: JVM / Java runtime dependency consumed transitively by the
  planned WIE integration
- License: MIT
- Copyright: Copyright 2020 Inseok Lee
- Current 0.0.2-T001 status: planned / not yet linked

MIT permission and warranty terms are provided by the upstream project's
LICENSE and must be preserved in M32 distributions that include substantial
portions of RustJava.

## 3. Components not yet selected

The following categories deliberately have no final component selected at 0.0.2-T001:

- M32 UI fonts
- MIDI software synthesizer
- SoundFont
- PCM/audio output library
- image/audio preview codecs
- installer/runtime packaging dependencies

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
inventory must be compared against this file. A notice for a planned but
unused component may remain clearly marked as planned; a notice for an
actually shipped third-party component may not be omitted.

This document is an engineering compliance record, not legal advice.
