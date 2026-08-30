//! Stable M32-facing emulator backend contracts.
//!
//! This crate must remain independent from any concrete emulator implementation.
//! WIE-specific types belong behind `m32-wie-adapter`.

use std::{error::Error, fmt};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostServiceKind {
    Display,
    Clock,
    Database,
    Filesystem,
    Audio,
    Stdout,
    Stderr,
    Exit,
    Vibration,
}

impl HostServiceKind {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::Clock => "clock",
            Self::Database => "database",
            Self::Filesystem => "filesystem",
            Self::Audio => "audio",
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::Exit => "exit",
            Self::Vibration => "vibration",
        }
    }
}

pub trait EmulatorBackend: Send + Sync {
    fn descriptor(&self) -> BackendDescriptor;
    fn required_host_services(&self) -> &'static [HostServiceKind];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionState {
    Ready,
    Running,
    Faulted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionErrorCode {
    BackendTickFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmulatorSessionError {
    pub code: SessionErrorCode,
    pub message: String,
}

impl EmulatorSessionError {
    #[must_use]
    pub fn new(code: SessionErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for EmulatorSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl Error for EmulatorSessionError {}

pub trait EmulatorSession {
    fn backend(&self) -> BackendDescriptor;
    fn state(&self) -> SessionState;
    fn tick(&mut self) -> Result<(), EmulatorSessionError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SyntheticBackend;

    impl EmulatorBackend for SyntheticBackend {
        fn descriptor(&self) -> BackendDescriptor {
            BackendDescriptor::new("synthetic", "Synthetic Backend", "test-revision")
        }

        fn required_host_services(&self) -> &'static [HostServiceKind] {
            &[]
        }
    }

    struct SyntheticSession {
        state: SessionState,
    }

    impl EmulatorSession for SyntheticSession {
        fn backend(&self) -> BackendDescriptor {
            BackendDescriptor::new("synthetic", "Synthetic Backend", "test-revision")
        }

        fn state(&self) -> SessionState {
            self.state
        }

        fn tick(&mut self) -> Result<(), EmulatorSessionError> {
            self.state = SessionState::Running;
            Ok(())
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
        assert!(backend.required_host_services().is_empty());
    }

    #[test]
    fn host_service_ids_are_stable() {
        let services = [
            (HostServiceKind::Display, "display"),
            (HostServiceKind::Clock, "clock"),
            (HostServiceKind::Database, "database"),
            (HostServiceKind::Filesystem, "filesystem"),
            (HostServiceKind::Audio, "audio"),
            (HostServiceKind::Stdout, "stdout"),
            (HostServiceKind::Stderr, "stderr"),
            (HostServiceKind::Exit, "exit"),
            (HostServiceKind::Vibration, "vibration"),
        ];

        for (service, expected_id) in services {
            assert_eq!(service.id(), expected_id);
        }
    }

    #[test]
    fn host_service_kinds_are_distinct() {
        let services = [
            HostServiceKind::Display,
            HostServiceKind::Clock,
            HostServiceKind::Database,
            HostServiceKind::Filesystem,
            HostServiceKind::Audio,
            HostServiceKind::Stdout,
            HostServiceKind::Stderr,
            HostServiceKind::Exit,
            HostServiceKind::Vibration,
        ];

        for left in 0..services.len() {
            for right in left + 1..services.len() {
                assert_ne!(services[left], services[right]);
            }
        }
    }

    #[test]
    fn session_state_contract_is_explicit() {
        assert_ne!(SessionState::Ready, SessionState::Running);
        assert_ne!(SessionState::Running, SessionState::Faulted);
        assert_ne!(SessionState::Ready, SessionState::Faulted);
    }

    #[test]
    fn session_error_keeps_stable_code_and_message() {
        let error = EmulatorSessionError::new(SessionErrorCode::BackendTickFailed, "synthetic failure");

        assert_eq!(error.code, SessionErrorCode::BackendTickFailed);
        assert_eq!(error.message, "synthetic failure");
        assert!(error.to_string().contains("BackendTickFailed"));
    }

    #[test]
    fn session_trait_is_implementation_agnostic() {
        let mut session = SyntheticSession {
            state: SessionState::Ready,
        };

        assert_eq!(session.state(), SessionState::Ready);
        assert_eq!(session.backend().id, "synthetic");

        session.tick().expect("synthetic tick must succeed");

        assert_eq!(session.state(), SessionState::Running);
    }
}
