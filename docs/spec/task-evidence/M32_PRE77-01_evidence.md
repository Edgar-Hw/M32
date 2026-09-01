# M32 PRE77-01 Evidence — Clean Reproducible Baseline

Status: probe PASS / final clean acceptance is determined by `scripts/verify-pre77-01-clean-baseline.ps1`.

## Purpose

PRE77-01 establishes one clean, reproducible repository baseline before productization work continues. It does not reopen or renumber the completed 76/253 roadmap tasks.

## Recorded probe

The baseline probe completed successfully on 2026-09-01 with:

- repository HEAD: `8e16099b9545e4820c6ab131158dbf55f9b8587b`
- branch: `main`
- Cargo.lock SHA-256: `44beca1082de30191d5035ccc3b7cfb9e6c2f350e5e1121b9e678c9d242dd993`
- rustc: `1.98.0 (88d9e12ae 2026-08-18)`
- cargo: `1.98.0 (797e8a9bc 2026-08-05)`
- host: `x86_64-pc-windows-msvc`
- pinned WIE revision resolved as `f0513eb758c02736981f545ad030eed937d55f3e`
- WIE/RustJava/SMAF Cargo checkout local modifications: `0`
- existing M32 0.1.0 First Playable version-close verifier: `PASS`
- consecutive locked desktop build #1 SHA-256: `3be6cf1c9af72546a0e44bbd80add98dcfbf11af002914a71baf99b220089f95`
- consecutive locked desktop build #2 SHA-256: `3be6cf1c9af72546a0e44bbd80add98dcfbf11af002914a71baf99b220089f95`

The two unchanged locked builds produced the same executable hash.

## Final acceptance gate

PRE77-01 is closed only when the repository contains the PRE77-01 probe/evidence/verifier files in a committed state and:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify-pre77-01-clean-baseline.ps1
```

passes from an empty Git working tree.

The verifier requires:

1. empty tracked/untracked Git status before verification,
2. PRE77-01 artifacts already tracked by Git,
3. the full 0.1.0 canonical version-close chain,
4. locked dependency graph resolution,
5. no local WIE/RustJava/SMAF Cargo checkout modifications,
6. two consecutive `cargo build -p m32-desktop --locked` runs with identical `m32.exe` SHA-256,
7. empty Git status after verification.

Logs are written only below `target/pre77/`.

## Policy locked by this gate

- Do not patch Cargo git checkout directories.
- Do not use `cargo clean` as a normal game-test workflow.
- Preserve the existing desktop/composition → emulator-api → m32-wie-adapter → WIE boundary.
- PRE77 gates do not consume or renumber the official 253 roadmap tasks.
