# M32 0.0.3-T006 FIX1 — advance executor time for MIDP repaint event loop

## Trigger

The first real tick-until-frame test timed out:

```text
first guest-generated frame was not captured within 512 ticks;
redraws=1,
presents=0
```

This proves:

```text
PaintMidlet.startApp
-> Display.setCurrent
-> request_redraw
```

succeeded.

The failure occurs after redraw request generation and before `Canvas.paint()` reaches the M32
display host.

## Exact Root Cause

Pinned MIDP `EventQueue::getNextEvent()` polls the backend event queue.

When the queue is empty it executes:

```text
system.sleep(16).await
```

Pinned WIE `SleepFuture::new()` registers the current executor task to wake at:

```text
executor.last_now + 16ms
```

Pinned executor polling only resumes a sleeping task when:

```text
platform.now() >= wakeup
```

The existing M32 test platform uses:

```text
FixedClock(1_725_123_456_789)
```

That clock is intentionally constant.

Positive boot does not require the spawned event loop to wake from its 16ms idle sleep, so the fixed
clock was sufficient through T005.

T006 does require the event loop to wake after the host pumps `Event::Redraw`.

The sequence was therefore:

```text
event loop sees empty queue
-> sleep until fixed_time + 16ms
-> host observes redraw request
-> host pushes Event::Redraw
-> later ticks still report the same fixed_time
-> sleeping EventQueue task never wakes
-> Redraw remains unconsumed
-> presents=0 forever
```

The observed:

```text
redraws=1
presents=0
```

matches this behavior exactly.

## FIX1

Add test-only:

```text
DeterministicAdvancingClock
```

For T006 paint runtime only:

```text
start = 1_725_123_456_789
step  = 1ms per epoch_millis() call
```

This preserves deterministic time while allowing WIE executor sleep deadlines to expire.

Existing `FixedClock` behavior is unchanged and remains used by tests that require an exact stable
epoch value.

## Regression

Added:

```text
deterministic_advancing_clock_moves_forward_for_sleeping_tasks
```

Expected sequence:

```text
1725123456789
1725123456790
1725123456791
```

## Scope

Test-only M32 adapter code.

No production clock contract change.

No public API change.

No WIE source change.

No RustJava source change.

No guest JAD/JAR/class change.

No dependency or Cargo.lock change.

## Expected T006 behavior

With advancing executor time:

```text
Display.setCurrent
-> request_redraw
-> host pumps Event::Redraw
-> sleeping MIDP event loop reaches its 16ms deadline
-> EventQueue consumes Redraw
-> RepaintEvent
-> Canvas.paint
-> WIE screen.paint
-> M32 present_rgba8
-> first frame captured
```

## Status

```text
0.0.3-T006 = FIX1 / IN_PROGRESS
```
