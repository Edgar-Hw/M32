# M32 Emulator API Boundary

Status: LOCKED BASELINE
Task: `0.0.2-T002`
Schema: `1`

## 1. 목적

M32의 application-facing emulator contract와 concrete WIE implementation을 분리한다.

Dependency direction:

```text
M32 upper layers
       ↓
m32-emulator-api
       ↑
m32-wie-adapter
       ↓
wie_backend
```

`m32-emulator-api` must never depend on WIE.

## 2. T002 Contract

T002 introduces the smallest stable backend identity contract.

```text
BackendDescriptor
- id
- display_name
- upstream_revision

EmulatorBackend
- descriptor()
```

API schema:

```text
EMULATOR_API_SCHEMA_VERSION=1
```

T002 deliberately does not define game loading, ticking, screen frames, input, audio, filesystem,
or save-state methods yet. Those contracts are introduced by later Core Adapter Tasks when their
ownership and error semantics are fixed.

## 3. WIE Adapter

`WieBackendAdapter` implements `EmulatorBackend`.

Locked identity:

```text
id=wie
display_name=WIE
upstream_revision=f0513eb758c02736981f545ad030eed937d55f3e
```

This is the first M32 API value produced by the concrete WIE adapter.

## 4. WIE Type Leakage Rule

Forbidden examples outside `m32-wie-adapter`:

```rust
pub use wie_backend::Emulator;
pub use wie_backend::Event;
pub use wie_backend::KeyCode;
```

M32 upper layers must not make WIE types part of their public signatures.

When later Tasks need events, keys, frames, errors, or session objects, M32-owned equivalents are
defined in `m32-emulator-api` and translated inside the adapter.

## 5. Why the Contract Starts Small

The pinned WIE backend currently exposes its own:

- `Emulator` trait with event/tick methods;
- `Platform` abstraction;
- `Screen` abstraction;
- `Event` and `KeyCode` types.

T002 does not mirror these APIs mechanically. M32 has different long-term requirements for
diagnostics, display modes, input policy, saves, replay, and multiple legacy platforms.

Locking only backend identity first avoids accidentally turning current WIE implementation details
into the permanent M32 architecture.

## 6. Verification

Canonical boundary verification:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify-emulator-api-boundary.ps1
```

It verifies:

- `m32-emulator-api` has no WIE dependency;
- `m32-wie-adapter` directly depends on `m32-emulator-api`;
- `m32-wie-adapter` directly depends on `wie_backend`;
- `m32-desktop` has no direct WIE dependency.

## 7. T002 Non-Goals

T002 does not:

- instantiate WIE;
- define a launch request;
- create a session;
- tick an emulator;
- translate key events;
- expose framebuffer data;
- route audio;
- implement platform/filesystem services.
