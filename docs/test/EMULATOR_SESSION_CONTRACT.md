# M32 Emulator Session Contract

Status: LOCKED BASELINE
Task: `0.0.2-T003`
Schema: `1`

## 1. 목적

Concrete emulator runtime의 execution step을 M32-owned session contract 뒤에 둔다.

The pinned WIE backend exposes:

```text
Emulator::handle_event(...)
Emulator::tick() -> Result<()>
```

T003 integrates only the `tick()` side of that boundary.

Input/event translation remains out of scope until the dedicated Core Adapter input-event Task.

## 2. M32 Session Contract

`m32-emulator-api` owns:

```text
SessionState
- Ready
- Running
- Faulted

SessionErrorCode
- BackendTickFailed

EmulatorSessionError
- code
- message

EmulatorSession
- backend()
- state()
- tick()
```

These types do not depend on WIE.

## 3. State Semantics

`Ready`:

- a concrete runtime has been created;
- M32 has not yet observed a successful execution tick.

`Running`:

- at least one M32-requested tick completed successfully.

`Faulted`:

- a backend tick returned an error.

T003 does not define `Stopped`, `Paused`, or `Exited` because the current backend boundary does not
yet provide a stable M32-owned termination/host-control contract.

Those states must not be invented from WIE implementation details.

## 4. Error Semantics

T003 maps any WIE `tick()` error to the stable M32 code:

```text
BackendTickFailed
```

The backend's display string is kept as diagnostic detail in `message`.

The WIE error enum itself is not exposed from `m32-emulator-api`.

Later compatibility/Doctor Tasks may refine backend failure categorization, but callers must not
depend on WIE error variants directly.

## 5. WIE Session Wrapper

`m32-wie-adapter::WieSession` owns:

```text
Box<dyn wie_backend::Emulator>
```

internally.

Its public M32-facing behavior is only through `EmulatorSession`.

The concrete WIE emulator object cannot be constructed by M32 upper layers because T003 exposes no
public constructor that accepts a WIE type.

A later factory/launch Task creates `WieSession` internally after platform and game-loading
ownership are fixed.

## 6. Threading

`EmulatorSession` deliberately has no `Send` or `Sync` supertrait in schema v1.

The emulator runtime is expected to be driven from an owning execution context, and T003 does not
assume that all concrete WIE runtime objects are thread-transferable.

Thread ownership is fixed by a later runtime orchestration Task.

## 7. Non-Goals

T003 does not:

- instantiate J2ME/SKT/LGT runtimes;
- load game files;
- create a WIE `Platform`;
- translate WIE `Event`;
- map `KeyCode`;
- expose a framebuffer;
- route audio;
- implement pause/stop/restart;
- classify individual `WieError` variants.

## 8. Acceptance

- `m32-emulator-api` session tests pass;
- `WieSession` implements `EmulatorSession`;
- WIE tick errors map to `BackendTickFailed`;
- no WIE dependency is added to `m32-emulator-api`;
- existing T002 dependency boundary remains valid;
- all workspace quality gates pass.
