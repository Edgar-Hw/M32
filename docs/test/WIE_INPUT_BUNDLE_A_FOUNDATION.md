# M32 0.0.4 Input — Bundle A (T001-T003)

Status: IMPLEMENTATION BASELINE

This bundle changes the execution unit only. Individual Task IDs remain tracked.

```text
0.0.4-T001  Stable M32Key / GuestInputEvent Contract
0.0.4-T002  EmulatorSession Input Dispatch Seam
0.0.4-T003  Exact M32 -> pinned WIE Key/Event Mapping
```

## T001 — Stable M32Key / GuestInputEvent Contract

`m32-emulator-api` now owns the backend-agnostic guest keypad vocabulary.

Exact 24-key surface:

```text
Up Down Left Right Ok
LeftSoft RightSoft
Clear Call Hangup
VolumeUp VolumeDown
Num0 Num1 Num2 Num3 Num4 Num5 Num6 Num7 Num8 Num9
Hash Star
```

The event phases are:

```text
KeyDown
KeyUp
KeyRepeat
```

No WIE type leaks through this API.

## T002 — EmulatorSession Input Dispatch Seam

The stable session contract adds:

```rust
fn handle_input(&mut self, event: GuestInputEvent);
```

Input dispatch is intentionally separate from `tick()`.

The contract is currently infallible because the pinned WIE `Emulator::handle_event(Event)` surface
is itself infallible.

A later task may add higher-level policy around session state, held keys, repeat, and limits. T002
does not invent those policies early.

## T003 — Exact M32 -> pinned WIE Mapping

Pinned WIE revision:

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

Pinned backend `KeyCode` contains the same 24 feature-phone key concepts.

M32 maps every key explicitly, with no integer cast and no catch-all branch.

Event phases map:

```text
GuestInputEvent::KeyDown   -> wie_backend::Event::Keydown
GuestInputEvent::KeyUp     -> wie_backend::Event::Keyup
GuestInputEvent::KeyRepeat -> wie_backend::Event::Keyrepeat
```

`WieSession::handle_input()` forwards only through this mapping.

## Pinned MIDP Runtime Path

Pinned WIE MIDP EventQueue converts:

```text
Event::Keydown   -> KeyboardEventType::KeyPressed
Event::Keyup     -> KeyboardEventType::KeyReleased
Event::Keyrepeat -> KeyboardEventType::KeyRepeated
```

and maps WIE key codes to MIDP integer codes before calling `Display.handleKeyEvent`.

The current Canvas then dispatches those phases virtually to:

```text
Canvas.keyPressed(int)
Canvas.keyReleased(int)
Canvas.keyRepeated(int)
```

Bundle A stops at the backend event boundary.

A deterministic Java Canvas that proves those virtual callbacks is Bundle B scope.

## Non-goals

Bundle A does not yet implement:

- desktop keyboard bindings;
- repeat delay/rate policy;
- six-held-key limit;
- a Java key-observer MIDlet;
- actual keyPressed/keyReleased/keyRepeated guest callback proof;
- Auto Keypad.

## Expected Test Counts

Before Bundle A:

```text
m32-emulator-api 29
m32-wie-adapter  58
workspace min   101
```

Bundle A adds:

```text
API tests       +3
adapter tests   +3
```

Expected:

```text
m32-emulator-api 32
m32-wie-adapter  61
workspace min   107
```

No dependency or Cargo.lock change is expected.
