pub const PRODUCT_SPEC_VERSION: &str = "1.0.0";
pub const SPEC_BUNDLE_VERSION: &str = "1.0.1";
pub const WIE_BASELINE_COMMIT: &str = "f0513eb758c02736981f545ad030eed937d55f3e";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildInfo {
    pub app_version: &'static str,
    pub product_spec_version: &'static str,
    pub spec_bundle_version: &'static str,
    pub git_commit: &'static str,
    pub wie_commit: &'static str,
    pub rust_version: &'static str,
    pub target: &'static str,
    pub build_profile: &'static str,
}

impl BuildInfo {
    #[must_use]
    pub const fn current() -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION"),
            product_spec_version: PRODUCT_SPEC_VERSION,
            spec_bundle_version: SPEC_BUNDLE_VERSION,
            git_commit: env!("M32_GIT_COMMIT"),
            wie_commit: WIE_BASELINE_COMMIT,
            rust_version: env!("M32_RUST_VERSION"),
            target: env!("M32_BUILD_TARGET"),
            build_profile: env!("M32_BUILD_PROFILE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_spec_baselines_match_locked_values() {
        let info = BuildInfo::current();

        assert_eq!(info.product_spec_version, "1.0.0");
        assert_eq!(info.spec_bundle_version, "1.0.1");
        assert_eq!(info.wie_commit, "f0513eb758c02736981f545ad030eed937d55f3e");
    }

    #[test]
    fn compile_time_build_fields_are_present() {
        let info = BuildInfo::current();

        assert!(!info.app_version.is_empty());
        assert!(!info.git_commit.is_empty());
        assert!(!info.rust_version.is_empty());
        assert!(!info.target.is_empty());
        assert!(!info.build_profile.is_empty());
    }

    #[test]
    fn pinned_rust_and_target_are_visible() {
        let info = BuildInfo::current();

        assert!(info.rust_version.starts_with("rustc 1.98.0 "));
        assert_eq!(info.target, "x86_64-pc-windows-msvc");
    }
}
