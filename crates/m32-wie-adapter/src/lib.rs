//! Adapter boundary between M32 and the pinned WIE upstream.
//!
//! WIE backend types stay private to this crate. M32-facing callers consume contracts from
//! `m32-emulator-api`.

use std::{fmt::Display, sync::Arc};

use m32_emulator_api::{
    BackendDescriptor, ClockHost, DisplayHost, DisplayHostError, DisplaySize, EmulatorBackend, EmulatorSession,
    EmulatorSessionError, ExitHost, GuestOutputHost, HostServiceKind, RgbaFrame, SessionErrorCode, SessionState,
    VibrationHost,
};
use wie_util::WieError;

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

pub struct WieBasicHostBridge {
    clock: Arc<dyn ClockHost>,
    output: Arc<dyn GuestOutputHost>,
    exit: Arc<dyn ExitHost>,
    vibration: Arc<dyn VibrationHost>,
}

impl WieBasicHostBridge {
    pub fn new(
        clock: Arc<dyn ClockHost>,
        output: Arc<dyn GuestOutputHost>,
        exit: Arc<dyn ExitHost>,
        vibration: Arc<dyn VibrationHost>,
    ) -> Self {
        Self {
            clock,
            output,
            exit,
            vibration,
        }
    }

    #[must_use]
    pub fn epoch_millis(&self) -> u64 {
        self.clock.epoch_millis()
    }

    pub fn write_stdout(&self, bytes: &[u8]) {
        self.output.write_stdout(bytes);
    }

    pub fn write_stderr(&self, bytes: &[u8]) {
        self.output.write_stderr(bytes);
    }

    pub fn request_exit(&self) {
        self.exit.request_exit();
    }

    pub fn vibrate(&self, duration_ms: u64, intensity: u8) {
        self.vibration.vibrate(duration_ms, intensity);
    }
}

pub struct WieScreenAdapter {
    host: Arc<dyn DisplayHost>,
}

impl WieScreenAdapter {
    #[must_use]
    pub fn new(host: Arc<dyn DisplayHost>) -> Self {
        Self { host }
    }
}

impl wie_backend::Screen for WieScreenAdapter {
    fn resize(&self, width: u32, height: u32) -> wie_util::Result<()> {
        self.host
            .resize(DisplaySize::new(width, height))
            .map_err(map_display_host_error)
    }

    fn request_redraw(&self) -> wie_util::Result<()> {
        self.host.request_redraw().map_err(map_display_host_error)
    }

    fn paint(&self, image: &dyn wie_backend::canvas::Image) {
        let Some(frame) = copy_wie_image_to_rgba8(image) else {
            tracing::error!(
                target: "m32::display",
                event = "wie_frame_dimension_overflow",
                width = image.width(),
                height = image.height(),
                "WIE frame dimensions overflow M32 RGBA8 frame size"
            );
            return;
        };

        if let Err(error) = self.host.present_rgba8(frame) {
            tracing::error!(
                target: "m32::display",
                event = "wie_frame_present_failed",
                error_code = ?error.code,
                "M32 display host rejected WIE frame"
            );
        }
    }

    fn width(&self) -> u32 {
        self.host.size().width
    }

    fn height(&self) -> u32 {
        self.host.size().height
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

fn map_display_host_error(error: DisplayHostError) -> WieError {
    WieError::FatalError(format!("M32 display host failure: {error}"))
}

fn copy_wie_image_to_rgba8(image: &dyn wie_backend::canvas::Image) -> Option<RgbaFrame> {
    let size = DisplaySize::new(image.width(), image.height());
    let byte_len = size.rgba8_byte_len()?;
    let mut pixels = Vec::with_capacity(byte_len);

    for y in 0..image.height() {
        for x in 0..image.width() {
            let color = image.get_pixel(x as i32, y as i32);
            pixels.extend_from_slice(&[color.r, color.g, color.b, color.a]);
        }
    }

    RgbaFrame::try_new(size, pixels).ok()
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        sync::{Arc, Mutex},
    };

    use super::*;
    use m32_emulator_api::DisplayHostErrorCode;
    use wie_backend::{Screen, canvas::Color};

    #[derive(Default)]
    struct RecordingDisplayHost {
        size: Mutex<DisplaySize>,
        redraw_count: Mutex<u32>,
        last_frame: Mutex<Option<RgbaFrame>>,
    }

    impl DisplayHost for RecordingDisplayHost {
        fn resize(&self, size: DisplaySize) -> Result<(), DisplayHostError> {
            *self.size.lock().expect("size mutex poisoned") = size;
            Ok(())
        }

        fn request_redraw(&self) -> Result<(), DisplayHostError> {
            let mut count = self.redraw_count.lock().expect("redraw mutex poisoned");
            *count += 1;
            Ok(())
        }

        fn present_rgba8(&self, frame: RgbaFrame) -> Result<(), DisplayHostError> {
            *self.last_frame.lock().expect("frame mutex poisoned") = Some(frame);
            Ok(())
        }

        fn size(&self) -> DisplaySize {
            *self.size.lock().expect("size mutex poisoned")
        }
    }

    struct SyntheticWieImage;

    impl wie_backend::canvas::Image for SyntheticWieImage {
        fn width(&self) -> u32 {
            2
        }

        fn height(&self) -> u32 {
            1
        }

        fn bytes_per_pixel(&self) -> u32 {
            4
        }

        fn get_pixel(&self, x: i32, y: i32) -> Color {
            assert_eq!(y, 0);

            match x {
                0 => Color {
                    a: 255,
                    r: 10,
                    g: 20,
                    b: 30,
                },
                1 => Color {
                    a: 128,
                    r: 40,
                    g: 50,
                    b: 60,
                },
                _ => panic!("unexpected x coordinate"),
            }
        }

        fn raw(&self) -> Cow<'_, [u8]> {
            Cow::Borrowed(&[])
        }

        fn colors(&self) -> Vec<Color> {
            vec![self.get_pixel(0, 0), self.get_pixel(1, 0)]
        }
    }

    struct FixedClock(u64);

    impl ClockHost for FixedClock {
        fn epoch_millis(&self) -> u64 {
            self.0
        }
    }

    #[derive(Default)]
    struct RecordingOutput {
        stdout: Mutex<Vec<u8>>,
        stderr: Mutex<Vec<u8>>,
    }

    impl GuestOutputHost for RecordingOutput {
        fn write_stdout(&self, bytes: &[u8]) {
            self.stdout
                .lock()
                .expect("stdout mutex poisoned")
                .extend_from_slice(bytes);
        }

        fn write_stderr(&self, bytes: &[u8]) {
            self.stderr
                .lock()
                .expect("stderr mutex poisoned")
                .extend_from_slice(bytes);
        }
    }

    #[derive(Default)]
    struct RecordingExit {
        count: Mutex<u32>,
    }

    impl ExitHost for RecordingExit {
        fn request_exit(&self) {
            let mut count = self.count.lock().expect("exit mutex poisoned");
            *count += 1;
        }
    }

    #[derive(Default)]
    struct RecordingVibration {
        last: Mutex<Option<(u64, u8)>>,
    }

    impl VibrationHost for RecordingVibration {
        fn vibrate(&self, duration_ms: u64, intensity: u8) {
            *self.last.lock().expect("vibration mutex poisoned") = Some((duration_ms, intensity));
        }
    }

    fn recording_basic_bridge(
        epoch_millis: u64,
    ) -> (
        WieBasicHostBridge,
        Arc<RecordingOutput>,
        Arc<RecordingExit>,
        Arc<RecordingVibration>,
    ) {
        let output = Arc::new(RecordingOutput::default());
        let exit = Arc::new(RecordingExit::default());
        let vibration = Arc::new(RecordingVibration::default());

        let bridge = WieBasicHostBridge::new(
            Arc::new(FixedClock(epoch_millis)),
            output.clone(),
            exit.clone(),
            vibration.clone(),
        );

        (bridge, output, exit, vibration)
    }

    #[test]
    fn basic_host_bridge_maps_epoch_millis_to_wie_instant() {
        let (bridge, _, _, _) = recording_basic_bridge(1_725_000_123_456);
        let instant = wie_backend::Instant::from_epoch_millis(bridge.epoch_millis());

        assert_eq!(instant.raw(), 1_725_000_123_456);
    }

    #[test]
    fn basic_host_bridge_forwards_guest_output_as_raw_bytes() {
        let (bridge, output, _, _) = recording_basic_bridge(0);

        bridge.write_stdout(&[0x00, 0xFF, b'M']);
        bridge.write_stderr(&[0x80, b'E']);

        assert_eq!(
            *output.stdout.lock().expect("stdout mutex poisoned"),
            vec![0x00, 0xFF, b'M']
        );
        assert_eq!(*output.stderr.lock().expect("stderr mutex poisoned"), vec![0x80, b'E']);
    }

    #[test]
    fn basic_host_bridge_forwards_guest_exit_request() {
        let (bridge, _, exit, _) = recording_basic_bridge(0);

        bridge.request_exit();

        assert_eq!(*exit.count.lock().expect("exit mutex poisoned"), 1);
    }

    #[test]
    fn basic_host_bridge_forwards_vibration_values() {
        let (bridge, _, _, vibration) = recording_basic_bridge(0);

        bridge.vibrate(900, 170);

        assert_eq!(
            *vibration.last.lock().expect("vibration mutex poisoned"),
            Some((900, 170))
        );
    }

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
    fn wie_screen_adapter_implements_pinned_screen_contract() {
        fn assert_screen<T: wie_backend::Screen>() {}

        assert_screen::<WieScreenAdapter>();
    }

    #[test]
    fn wie_screen_resize_and_redraw_delegate_to_m32_host() {
        let host = Arc::new(RecordingDisplayHost::default());
        let screen = WieScreenAdapter::new(host.clone());

        screen.resize(240, 320).expect("resize must succeed");
        screen.request_redraw().expect("redraw request must succeed");

        assert_eq!(screen.width(), 240);
        assert_eq!(screen.height(), 320);
        assert_eq!(*host.redraw_count.lock().expect("redraw mutex poisoned"), 1);
    }

    #[test]
    fn wie_screen_paint_converts_colors_to_canonical_rgba8() {
        let host = Arc::new(RecordingDisplayHost::default());
        let screen = WieScreenAdapter::new(host.clone());

        screen.paint(&SyntheticWieImage);

        let frame = host
            .last_frame
            .lock()
            .expect("frame mutex poisoned")
            .clone()
            .expect("frame must be presented");

        assert_eq!(frame.size, DisplaySize::new(2, 1));
        assert_eq!(frame.pixels, vec![10, 20, 30, 255, 40, 50, 60, 128]);
    }

    #[test]
    fn display_host_error_maps_to_wie_fatal_error() {
        let error = map_display_host_error(DisplayHostError::new(
            DisplayHostErrorCode::ResizeFailed,
            "synthetic resize failure",
        ));

        assert!(matches!(error, WieError::FatalError(_)));
        assert!(error.to_string().contains("synthetic resize failure"));
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
