//! Synthetic fixture registry for M32 tests.
//!
//! Commercial game files must never be embedded in this crate.

/// Repository-relative path to the canonical fixture manifest.
pub const FIXTURE_MANIFEST_PATH: &str = "assets/fixtures/fixture-manifest.json";

/// Canonical manifest schema version introduced by `0.0.1-T007`.
pub const FIXTURE_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// IDs that must exist in the baseline T007 fixture manifest.
pub const BASELINE_FIXTURE_IDS: &[&str] = &[
    "j2me-hello-source",
    "j2me-input-source",
    "j2me-audio-source",
    "malformed-truncated-zip",
    "malformed-truncated-class",
    "malformed-truncated-png",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_fixture_ids_are_unique() {
        for (index, fixture_id) in BASELINE_FIXTURE_IDS.iter().enumerate() {
            assert!(
                !BASELINE_FIXTURE_IDS[..index].contains(fixture_id),
                "duplicate baseline fixture id: {fixture_id}"
            );
        }
    }

    #[test]
    fn baseline_fixture_ids_are_kebab_case_ascii() {
        for fixture_id in BASELINE_FIXTURE_IDS {
            assert!(!fixture_id.is_empty());
            assert!(!fixture_id.starts_with('-'));
            assert!(!fixture_id.ends_with('-'));
            assert!(
                fixture_id
                    .bytes()
                    .all(|byte| { byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' })
            );
        }
    }
}
