# M32 WIE Upstream Pin

Status: LOCKED BASELINE
Task: `0.0.2-T001`
Fixes: `FIX1 — transitive SMAF pin`, `FIX2 — tracing compatibility pin`

## 1. WIE Repository

```text
https://github.com/dlunch/wie.git
```

## 2. WIE Revision

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

M32 must not track WIE `main`, a branch name, or an unpinned WIE Git dependency.

## 3. First Integration Surface

T001 connects:

```text
m32-wie-adapter
        ↓
wie_backend
```

The desktop application does not yet instantiate or run a WIE emulator session.

## 4. Upstream Transitive Pin Problem

The pinned WIE `wie_backend/Cargo.toml` declares these Git dependencies without a revision:

```text
smaf
smaf_player
```

When WIE is consumed as a dependency of M32, WIE's nested `Cargo.lock` does not become the M32
workspace lockfile. M32 therefore cannot rely on that nested lockfile to reproduce WIE's transitive
Git dependency revision.

The WIE lockfile at the pinned WIE revision records both packages at:

```text
SMAF repository:
https://github.com/dlunch/smaf.git

SMAF revision:
8009d78512fd121609a841f31aa527bf2a4af456
```

T001 FIX1 treats that revision as the compatibility baseline for this WIE revision.

## 5. SMAF Minimal Vendoring

Canonical command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\vendor-smaf.ps1
```

The script checks out the exact SMAF revision into a temporary directory and copies only:

```text
third_party\smaf\
├─ LICENSE
├─ M32_UPSTREAM.json
├─ smaf\
│  ├─ Cargo.toml
│  └─ src\
└─ smaf_player\
   ├─ Cargo.toml
   └─ src\
```

It deliberately does not vendor:

- upstream Git history;
- `smaf_cli`;
- `test_data`;
- upstream tests;
- unrelated repository automation/configuration.

This keeps the workaround limited to the crates required by `wie_backend`.

## 6. Cargo Path Patch

The M32 workspace uses a top-level Cargo patch:

```toml
[patch."https://github.com/dlunch/smaf.git"]
smaf = { path = "third_party/smaf/smaf" }
smaf_player = { path = "third_party/smaf/smaf_player" }
```

This is necessary because Cargo patch configuration is applied from the root workspace and path
patches can replace transitive Git dependencies.

The vendored source is committed to M32 so CI and later development sessions do not depend on the
mutable SMAF default branch.


## 7. Tracing Compatibility Pin

The first T001 integration attempt after SMAF FIX1 exposed a second resolver conflict.

The pinned WIE revision declares:

```text
workspace tracing: ^0.1 with `attributes`
wie_util tracing-attributes: <0.1.29
```

The WIE lockfile at the pinned revision resolves this as:

```text
tracing = 0.1.41
tracing-attributes = 0.1.28
tracing-core = 0.1.36
```

M32 Foundation previously selected `tracing = 0.1.44`. That version requires
`tracing-attributes >=0.1.31` when the `attributes` feature is unified by WIE, so Cargo cannot
satisfy WIE's `<0.1.29` constraint.

T001 FIX2 therefore changes the M32 exact tracing baseline to:

```text
tracing = 0.1.41
```

`tracing-subscriber = 0.3.23` remains unchanged.

This is a compatibility correction made as part of the active WIE integration Task. It does not
change the locked WIE revision or the M32 logging API/behavior established by T009.

M32 must not remove WIE's `<0.1.29` safety constraint merely to retain a newer tracing patch
version, because the upstream constraint explicitly exists to avoid a no-std compile failure.


## 8. Boundary Rule

`m32-wie-adapter` must not publicly re-export WIE types as M32's stable application API.

The stable M32-facing contract belongs to:

```text
m32-emulator-api
```

Later Core Adapter Tasks translate between that contract and the pinned WIE API.

## 9. Cargo Lock

After T001 resolves successfully, the M32 root `Cargo.lock` is committed.

The resulting lockfile plus:

- WIE revision;
- vendored SMAF revision;
- M32 commit;

form the reproducible dependency baseline.

## 10. Verification

Canonical commands:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\vendor-smaf.ps1
powershell -ExecutionPolicy Bypass -File scripts\verify-wie-upstream.ps1
```

Verification confirms:

- exact WIE Git repository and revision;
- direct `m32-wie-adapter → wie_backend` dependency;
- exact SMAF provenance marker;
- `smaf` resolves from `third_party/smaf/smaf`;
- `smaf_player` resolves from `third_party/smaf/smaf_player`;
- no mutable SMAF Git HEAD remains in the resolved graph for these packages.

## 11. T001 Non-Goals

T001 still does not:

- boot a game;
- instantiate a WIE emulator;
- map M32 input;
- route audio;
- expose WIE types to M32 UI;
- integrate J2ME/SKT/LGT/WIPI launcher crates.
