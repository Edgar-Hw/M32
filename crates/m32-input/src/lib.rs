//! M32 guest input scheduling and held-key policy.

use m32_emulator_api::{GuestInputEvent, M32Key};

pub const KEY_REPEAT_DELAY_MS: u64 = 350;
pub const KEY_REPEAT_HZ: u64 = 12;
pub const MAX_HELD_GUEST_KEYS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyDownOutcome {
    Accepted,
    AlreadyHeld,
    HeldKeyLimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HeldKey {
    key: M32Key,
    pressed_at_ms: u64,
    repeats_emitted: u64,
}

#[derive(Debug, Default)]
pub struct GuestInputController {
    held: Vec<HeldKey>,
}

impl GuestInputController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn held_count(&self) -> usize {
        self.held.len()
    }

    pub fn is_held(&self, key: M32Key) -> bool {
        self.held.iter().any(|held| held.key == key)
    }

    pub fn key_down(&mut self, key: M32Key, now_ms: u64) -> (KeyDownOutcome, Option<GuestInputEvent>) {
        if self.is_held(key) {
            return (KeyDownOutcome::AlreadyHeld, None);
        }

        if self.held.len() >= MAX_HELD_GUEST_KEYS {
            return (KeyDownOutcome::HeldKeyLimitReached, None);
        }

        self.held.push(HeldKey {
            key,
            pressed_at_ms: now_ms,
            repeats_emitted: 0,
        });

        (KeyDownOutcome::Accepted, Some(GuestInputEvent::KeyDown(key)))
    }

    pub fn key_up(&mut self, key: M32Key) -> Option<GuestInputEvent> {
        let index = self.held.iter().position(|held| held.key == key)?;
        self.held.remove(index);
        Some(GuestInputEvent::KeyUp(key))
    }

    pub fn repeats_due(&mut self, now_ms: u64) -> Vec<GuestInputEvent> {
        let mut events = Vec::new();

        for held in &mut self.held {
            let elapsed_ms = now_ms.saturating_sub(held.pressed_at_ms);
            if elapsed_ms < KEY_REPEAT_DELAY_MS {
                continue;
            }

            let after_delay_ms = elapsed_ms - KEY_REPEAT_DELAY_MS;
            let total_due = 1_u64.saturating_add(after_delay_ms.saturating_mul(KEY_REPEAT_HZ) / 1000);
            let new_due = total_due.saturating_sub(held.repeats_emitted);

            for _ in 0..new_due {
                events.push(GuestInputEvent::KeyRepeat(held.key));
            }

            held.repeats_emitted = total_due;
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_policy_starts_at_350ms_and_tracks_12hz_without_drift() {
        let mut input = GuestInputController::new();

        assert_eq!(
            input.key_down(M32Key::Right, 1_000),
            (KeyDownOutcome::Accepted, Some(GuestInputEvent::KeyDown(M32Key::Right)))
        );

        assert!(input.repeats_due(1_349).is_empty());
        assert_eq!(
            input.repeats_due(1_350),
            vec![GuestInputEvent::KeyRepeat(M32Key::Right)]
        );

        assert!(input.repeats_due(1_433).is_empty());
        assert_eq!(
            input.repeats_due(1_434),
            vec![GuestInputEvent::KeyRepeat(M32Key::Right)]
        );

        assert_eq!(
            input.repeats_due(1_517),
            vec![GuestInputEvent::KeyRepeat(M32Key::Right)]
        );
        assert_eq!(
            input.repeats_due(1_600),
            vec![GuestInputEvent::KeyRepeat(M32Key::Right)]
        );
        assert_eq!(
            input.repeats_due(1_684),
            vec![GuestInputEvent::KeyRepeat(M32Key::Right)]
        );
    }

    #[test]
    fn repeat_policy_catches_up_from_press_origin_without_schedule_drift() {
        let mut input = GuestInputController::new();
        input.key_down(M32Key::Num5, 10_000);

        assert_eq!(
            input.repeats_due(10_600),
            vec![
                GuestInputEvent::KeyRepeat(M32Key::Num5),
                GuestInputEvent::KeyRepeat(M32Key::Num5),
                GuestInputEvent::KeyRepeat(M32Key::Num5),
                GuestInputEvent::KeyRepeat(M32Key::Num5),
            ]
        );

        assert!(input.repeats_due(10_600).is_empty());

        assert_eq!(
            input.repeats_due(10_684),
            vec![GuestInputEvent::KeyRepeat(M32Key::Num5)]
        );
    }

    #[test]
    fn release_stops_future_repeats_and_emits_one_key_up() {
        let mut input = GuestInputController::new();
        input.key_down(M32Key::Up, 0);

        assert_eq!(input.repeats_due(350), vec![GuestInputEvent::KeyRepeat(M32Key::Up)]);
        assert_eq!(input.key_up(M32Key::Up), Some(GuestInputEvent::KeyUp(M32Key::Up)));
        assert_eq!(input.held_count(), 0);
        assert!(input.repeats_due(10_000).is_empty());
        assert_eq!(input.key_up(M32Key::Up), None);
    }

    #[test]
    fn duplicate_key_down_does_not_duplicate_held_key_or_guest_press() {
        let mut input = GuestInputController::new();

        assert_eq!(
            input.key_down(M32Key::Ok, 100),
            (KeyDownOutcome::Accepted, Some(GuestInputEvent::KeyDown(M32Key::Ok)))
        );
        assert_eq!(input.key_down(M32Key::Ok, 200), (KeyDownOutcome::AlreadyHeld, None));

        assert_eq!(input.held_count(), 1);
        assert!(input.is_held(M32Key::Ok));
    }

    #[test]
    fn held_guest_key_limit_is_exactly_six_and_seventh_is_rejected() {
        let mut input = GuestInputController::new();
        let accepted = [
            M32Key::Up,
            M32Key::Down,
            M32Key::Left,
            M32Key::Right,
            M32Key::Ok,
            M32Key::Num5,
        ];

        for (index, key) in accepted.into_iter().enumerate() {
            assert_eq!(
                input.key_down(key, index as u64),
                (KeyDownOutcome::Accepted, Some(GuestInputEvent::KeyDown(key)))
            );
        }

        assert_eq!(input.held_count(), MAX_HELD_GUEST_KEYS);
        assert_eq!(
            input.key_down(M32Key::LeftSoft, 100),
            (KeyDownOutcome::HeldKeyLimitReached, None)
        );
        assert_eq!(input.held_count(), MAX_HELD_GUEST_KEYS);

        assert_eq!(input.key_up(M32Key::Left), Some(GuestInputEvent::KeyUp(M32Key::Left)));
        assert_eq!(input.held_count(), MAX_HELD_GUEST_KEYS - 1);

        assert_eq!(
            input.key_down(M32Key::LeftSoft, 101),
            (
                KeyDownOutcome::Accepted,
                Some(GuestInputEvent::KeyDown(M32Key::LeftSoft))
            )
        );
        assert_eq!(input.held_count(), MAX_HELD_GUEST_KEYS);
    }

    #[test]
    fn repeat_order_follows_stable_held_key_insertion_order() {
        let mut input = GuestInputController::new();
        input.key_down(M32Key::Left, 0);
        input.key_down(M32Key::Right, 0);
        input.key_down(M32Key::Num1, 0);

        assert_eq!(
            input.repeats_due(350),
            vec![
                GuestInputEvent::KeyRepeat(M32Key::Left),
                GuestInputEvent::KeyRepeat(M32Key::Right),
                GuestInputEvent::KeyRepeat(M32Key::Num1),
            ]
        );
    }
}
