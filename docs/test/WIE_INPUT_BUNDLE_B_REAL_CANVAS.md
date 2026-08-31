# M32 0.0.4 Input — Bundle B (T004-T007)

Status: IMPLEMENTATION BASELINE

```text
0.0.4-T004  Deterministic Key-observer MIDlet Fixture
0.0.4-T005  Real KeyDown -> Canvas.keyPressed Proof
0.0.4-T006  Real KeyUp/KeyRepeat Canvas Callback Proof
0.0.4-T007  Full 24-key MIDP Code Matrix End-to-End
```

## T004

Adds an M32-owned deterministic J2ME fixture:

```text
m32.KeyMidlet
m32.KeyCanvas
```

`KeyMidlet.startApp()` installs `KeyCanvas` through `Display.setCurrent`.

The Canvas overrides:

```text
keyPressed(int)
keyReleased(int)
keyRepeated(int)
```

and emits raw deterministic stdout tokens:

```text
M32_KEY_PRESSED:<code>;
M32_KEY_RELEASED:<code>;
M32_KEY_REPEATED:<code>;
```

The runtime JAR contains only:

```text
m32/KeyCanvas.class
m32/KeyMidlet.class
```

Compile-time MIDP stubs are not included in the guest runtime JAR.

## T005

The real runtime path must deliver:

```text
M32 KeyDown(Up)
-> WIE Event::Keydown(UP)
-> MIDP EventQueue KeyPressed
-> MIDP code 141
-> Display.handleKeyEvent
-> Canvas.handleKeyEvent
-> virtual KeyCanvas.keyPressed(141)
-> stdout M32_KEY_PRESSED:141;
```

## T006

Locks the other two phases through real guest callbacks:

```text
KeyUp(LeftSoft)
-> keyReleased(6)

KeyRepeat(Num7)
-> keyRepeated(55)
```

## T007

Runs all 24 M32 keys through the real Java Canvas `keyPressed(int)` callback and locks the pinned
MIDP integer matrix:

```text
Up 141        Down 146      Left 142      Right 145      Ok 148
LeftSoft 6    RightSoft 7   Clear 8       Call 10        Hangup -1
VolumeUp 13   VolumeDown 14

Num0 48       Num1 49       Num2 50       Num3 51       Num4 52
Num5 53       Num6 54       Num7 55       Num8 56       Num9 57

Hash 35       Star 42
```

## Fixture Hashes

```text
JAD
4edee7aaf35396e1965e1e5c6a2e4e0e9e22f3c94dd033638ca9be9e2aaf9825

JAR
be7cb8fa6933ac2b1ebd1303e3e9e549a8e731a6b0b0d9a2e44f630b22df7ca2

m32/KeyCanvas.class
6006ceae8a29921a40babeb41edd0dfa6184796c6e7c113bd34cc74c309be86f

m32/KeyMidlet.class
7a1e82b3f4d7fb850a7e0843db06077538594b45fc68c1bffe4dc01964154030
```

Both classes are Java 8 classfile major 52.

JAR is stored/no-compression, fixed timestamp 2000-01-01, no manifest.

## Expected Tests

Bundle A baseline:

```text
m32-emulator-api 32
m32-wie-adapter  61
workspace min   107
```

Bundle B adds five adapter tests:

```text
m32-wie-adapter  66
workspace min   112
```

API remains 32.

No dependency/Cargo.lock/WIE/RustJava change.
