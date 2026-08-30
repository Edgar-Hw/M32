//! Adapter boundary between M32 and the pinned WIE upstream.
//!
//! WIE backend types stay private to this crate. M32-facing callers consume contracts from
//! `m32-emulator-api`.

use std::fmt::Display;

use m32_emulator_api::{
    BackendDescriptor, EmulatorBackend, EmulatorSession, EmulatorSessionError, HostServiceKind, SessionErrorCode,
    SessionState,
};

pub const WIE_REPOSITORY: &str = "https://github.com/dlunch/wie.git";
pub const WIE_REVISION: &str = "f0513eb758c02736981f545ad030eed937d55f3e";
pub const WIE_BACKEND_ID: &str = "wie";
pub const WIE_BACKEND_DISPLAY_NAME: &str = "WIE";

pub const WIE_REQUIRED_HOST_SERVICES: &[HostServiceKind] = &[
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

    fn required_host_services(&self) -> &'static [HostServiceKind] {
        WIE_REQUIRED_HOST_SERVICES
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
    fn adapter_declares_all_wie_platform_host_requirements() {
        let backend: &dyn EmulatorBackend = &WieBackendAdapter::new();

        assert_eq!(
            backend.required_host_services(),
            &[
                HostServiceKind::Display,
                HostServiceKind::Clock,
                HostServiceKind::Database,
                HostServiceKind::Filesystem,
                HostServiceKind::Audio,
                HostServiceKind::Stdout,
                HostServiceKind::Stderr,
                HostServiceKind::Exit,
                HostServiceKind::Vibration,
            ]
        );
    }

    #[test]
    fn pinned_wie_platform_surface_is_still_compatible() {
        fn compile_probe(platform: &dyn wie_backend::Platform) {
            let _: &dyn wie_backend::Screen = platform.screen();
            let _: wie_backend::Instant = platform.now();
            let _: &dyn wie_backend::DatabaseRepository = platform.database_repository();
            let _: &dyn wie_backend::Filesystem = platform.filesystem();
            let _: Box<dyn wie_backend::AudioSink> = platform.audio_sink();

            platform.write_stdout(&[]);
            platform.write_stderr(&[]);
            platform.exit();
            platform.vibrate(0, 0);
        }

        let _ = compile_probe as fn(&dyn wie_backend::Platform);
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
