//! Adapter boundary between M32 and the pinned WIE upstream.
//!
//! WIE backend types stay private to this crate. M32-facing callers consume contracts from
//! `m32-emulator-api`.

use std::fmt::Display;

use m32_emulator_api::{
    BackendDescriptor, EmulatorBackend, EmulatorSession, EmulatorSessionError, SessionErrorCode, SessionState,
};

pub const WIE_REPOSITORY: &str = "https://github.com/dlunch/wie.git";
pub const WIE_REVISION: &str = "f0513eb758c02736981f545ad030eed937d55f3e";
pub const WIE_BACKEND_ID: &str = "wie";
pub const WIE_BACKEND_DISPLAY_NAME: &str = "WIE";

#[derive(Debug, Default, Clone, Copy)]
pub struct WieBackendAdapter;

impl WieBackendAdapter {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl EmulatorBackend for WieBackendAdapter {
    fn descriptor(&self) -> BackendDescriptor {
        wie_backend_descriptor()
    }
}

pub struct WieSession {
    emulator: Box<dyn wie_backend::Emulator>,
    state: SessionState,
}

impl EmulatorSession for WieSession {
    fn backend(&self) -> BackendDescriptor {
        wie_backend_descriptor()
    }

    fn state(&self) -> SessionState {
        self.state
    }

    fn tick(&mut self) -> Result<(), EmulatorSessionError> {
        match self.emulator.tick() {
            Ok(()) => {
                self.state = SessionState::Running;
                Ok(())
            }
            Err(error) => {
                self.state = SessionState::Faulted;
                Err(map_tick_error(error))
            }
        }
    }
}

fn wie_backend_descriptor() -> BackendDescriptor {
    BackendDescriptor::new(WIE_BACKEND_ID, WIE_BACKEND_DISPLAY_NAME, WIE_REVISION)
}

fn map_tick_error(error: impl Display) -> EmulatorSessionError {
    EmulatorSessionError::new(SessionErrorCode::BackendTickFailed, error.to_string())
}

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

    #[test]
    fn adapter_exposes_wie_identity_through_m32_api() {
        let backend: &dyn EmulatorBackend = &WieBackendAdapter::new();
        let descriptor = backend.descriptor();

        assert_eq!(descriptor.id, WIE_BACKEND_ID);
        assert_eq!(descriptor.display_name, WIE_BACKEND_DISPLAY_NAME);
        assert_eq!(descriptor.upstream_revision, WIE_REVISION);
    }

    #[test]
    fn wie_session_implements_m32_session_contract() {
        fn assert_session<T: EmulatorSession>() {}

        assert_session::<WieSession>();
    }

    #[test]
    fn wie_tick_error_maps_to_stable_m32_error_code() {
        let error = map_tick_error("synthetic WIE tick failure");

        assert_eq!(error.code, SessionErrorCode::BackendTickFailed);
        assert_eq!(error.message, "synthetic WIE tick failure");
    }
}
