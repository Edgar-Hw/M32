//! Stable M32-facing emulator backend contracts.
//!
//! This crate must remain independent from any concrete emulator implementation.
//! WIE-specific types belong behind `m32-wie-adapter`.

pub const EMULATOR_API_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BackendDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub upstream_revision: &'static str,
}

impl BackendDescriptor {
    #[must_use]
    pub const fn new(id: &'static str, display_name: &'static str, upstream_revision: &'static str) -> Self {
        Self {
            id,
            display_name,
            upstream_revision,
        }
    }
}

pub trait EmulatorBackend: Send + Sync {
    fn descriptor(&self) -> BackendDescriptor;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SyntheticBackend;

    impl EmulatorBackend for SyntheticBackend {
        fn descriptor(&self) -> BackendDescriptor {
            BackendDescriptor::new("synthetic", "Synthetic Backend", "test-revision")
        }
    }

    #[test]
    fn api_schema_is_version_one() {
        assert_eq!(EMULATOR_API_SCHEMA_VERSION, 1);
    }

    #[test]
    fn backend_descriptor_keeps_identity_fields() {
        let descriptor = BackendDescriptor::new("wie", "WIE", "abc123");

        assert_eq!(descriptor.id, "wie");
        assert_eq!(descriptor.display_name, "WIE");
        assert_eq!(descriptor.upstream_revision, "abc123");
    }

    #[test]
    fn backend_trait_is_implementation_agnostic() {
        let backend: &dyn EmulatorBackend = &SyntheticBackend;
        let descriptor = backend.descriptor();

        assert_eq!(descriptor.id, "synthetic");
        assert_eq!(descriptor.display_name, "Synthetic Backend");
        assert_eq!(descriptor.upstream_revision, "test-revision");
    }
}
