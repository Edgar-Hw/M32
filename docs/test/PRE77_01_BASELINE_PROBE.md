# M32 PRE77-01 Clean Baseline Probe

This bootstrap adds no production Rust code. It establishes the facts needed to close PRE77-01 without guessing the repository state.

It checks:

- current Git HEAD / branch / working tree
- Cargo.lock SHA-256
- Rust/Cargo toolchain identity
- locked Cargo metadata and git-sourced dependencies
- local Cargo WIE/RustJava/SMAF checkout contamination
- the existing 0.1.0 First Playable version-close chain
- two consecutive `cargo build -p m32-desktop --locked` runs
- stable `m32.exe` SHA-256 across those unchanged consecutive builds

The first run is intentionally **probe mode** because extracting this bundle itself creates new files. After the probe result is reviewed and the PRE77-01 evidence/verifier is finalized, the clean gate will be run with `-RequireClean` from a committed working tree.

No `cargo clean` is used. No Cargo checkout is modified.
