//! Adapter boundary between M32 and the pinned WIE upstream.
//!
//! `0.0.2-T001` intentionally does not re-export WIE backend types.
//! The public M32 emulator contract is defined separately by `m32-emulator-api`.

pub const WIE_REPOSITORY: &str = "https://github.com/dlunch/wie.git";
pub const WIE_REVISION: &str = "f0513eb758c02736981f545ad030eed937d55f3e";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_revision_matches_locked_baseline() {
        assert_eq!(WIE_REPOSITORY, "https://github.com/dlunch/wie.git");
        assert_eq!(WIE_REVISION, "f0513eb758c02736981f545ad030eed937d55f3e");
    }

    #[test]
    fn pinned_wie_backend_options_api_is_available() {
        let options = wie_backend::Options {
            enable_gdbserver: false,
            profile: None,
        };

        assert!(!options.enable_gdbserver);
        assert!(options.profile.is_none());
    }
}
