# M32 0.0.4 Input — Bundle C Functional Gate (T008-T009)

Status: IMPLEMENTATION BASELINE

```text
0.0.4-T008  Deterministic 350ms / 12Hz Key Repeat Policy
0.0.4-T009  Maximum Six Held Guest Keys
```

T010 version close is performed only after T008/T009 evidence is generated from a passing functional
gate.

## Ownership

`m32-input` owns host-side guest input timing and held-key policy.

`m32-emulator-api` owns the stable `M32Key` / `GuestInputEvent` vocabulary.

`m32-wie-adapter` remains responsible only for backend mapping and dispatch.

This keeps repeat policy out of the WIE-specific adapter.

## T008 Repeat Contract

Locked constants:

```text
initial repeat delay = 350ms
repeat frequency     = 12Hz
```

The first repeat is due exactly at:

```text
press_time + 350ms
```

Subsequent repeat count is calculated from the original repeat origin:

```text
1 + floor((elapsed_after_delay_ms * 12) / 1000)
```

This avoids cumulative deadline drift.

With integer-millisecond scheduling the first boundaries are:

```text
350ms
434ms
517ms
600ms
684ms
767ms
850ms
934ms
...
```

A delayed poll catches up from the original schedule rather than moving the schedule origin.

Releasing a held key immediately removes it from repeat scheduling.

## T009 Held-key Contract

Maximum simultaneously held guest keys:

```text
6
```

A seventh distinct key-down returns:

```text
HeldKeyLimitReached
```

and emits no `GuestInputEvent`.

Duplicate key-down for an already-held key returns:

```text
AlreadyHeld
```

and does not create a second guest press or reset its repeat origin.

After releasing one held key, a previously rejected key may be accepted.

Repeat emission order follows held-key insertion order for deterministic replay behavior.

## Dependency Boundary

`m32-input` adds one workspace-local dependency:

```text
m32-input -> m32-emulator-api
```

No external dependency is added.

Because the dependency edge is new, `Cargo.lock` may receive an intentional workspace package
dependency metadata update when Cargo runs.

No WIE or RustJava dependency/source change is expected.

## Tests

T008:

```text
repeat_policy_starts_at_350ms_and_tracks_12hz_without_drift
repeat_policy_catches_up_from_press_origin_without_schedule_drift
release_stops_future_repeats_and_emits_one_key_up
```

T009:

```text
duplicate_key_down_does_not_duplicate_held_key_or_guest_press
held_guest_key_limit_is_exactly_six_and_seventh_is_rejected
repeat_order_follows_stable_held_key_insertion_order
```

Expected:

```text
m32-input 6 tests
```

Bundle B baseline remains:

```text
m32-emulator-api 32
m32-wie-adapter  66
```

Minimum workspace unit-test total after T008/T009:

```text
9 + 3 + 32 + 6 + 2 + 66 = 118
```
