//! Stable M32-facing emulator backend contracts.
//!
//! This crate must remain independent from any concrete emulator implementation.
//! WIE-specific types belong behind `m32-wie-adapter`.

use std::{error::Error, fmt, future::Future, pin::Pin};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplaySize {
    pub width: u32,
    pub height: u32,
}

impl DisplaySize {
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn pixel_count(self) -> Option<usize> {
        match (self.width as usize).checked_mul(self.height as usize) {
            Some(value) => Some(value),
            None => None,
        }
    }

    #[must_use]
    pub const fn rgba8_byte_len(self) -> Option<usize> {
        match self.pixel_count() {
            Some(pixel_count) => pixel_count.checked_mul(4),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameValidationErrorCode {
    DimensionOverflow,
    ByteLengthMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameValidationError {
    pub code: FrameValidationErrorCode,
    pub expected_bytes: Option<usize>,
    pub actual_bytes: usize,
}

impl fmt::Display for FrameValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            FrameValidationErrorCode::DimensionOverflow => {
                write!(formatter, "display dimensions overflow RGBA8 byte length")
            }
            FrameValidationErrorCode::ByteLengthMismatch => write!(
                formatter,
                "RGBA8 byte length mismatch: expected {:?}, got {}",
                self.expected_bytes, self.actual_bytes
            ),
        }
    }
}

impl Error for FrameValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaFrame {
    pub size: DisplaySize,
    pub pixels: Vec<u8>,
}

impl RgbaFrame {
    pub fn try_new(size: DisplaySize, pixels: Vec<u8>) -> Result<Self, FrameValidationError> {
        let Some(expected_bytes) = size.rgba8_byte_len() else {
            return Err(FrameValidationError {
                code: FrameValidationErrorCode::DimensionOverflow,
                expected_bytes: None,
                actual_bytes: pixels.len(),
            });
        };

        if pixels.len() != expected_bytes {
            return Err(FrameValidationError {
                code: FrameValidationErrorCode::ByteLengthMismatch,
                expected_bytes: Some(expected_bytes),
                actual_bytes: pixels.len(),
            });
        }

        Ok(Self { size, pixels })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayHostErrorCode {
    ResizeFailed,
    RedrawFailed,
    PresentFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayHostError {
    pub code: DisplayHostErrorCode,
    pub message: String,
}

impl DisplayHostError {
    #[must_use]
    pub fn new(code: DisplayHostErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for DisplayHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl Error for DisplayHostError {}

pub trait DisplayHost: Send + Sync {
    fn resize(&self, size: DisplaySize) -> Result<(), DisplayHostError>;
    fn request_redraw(&self) -> Result<(), DisplayHostError>;
    fn present_rgba8(&self, frame: RgbaFrame) -> Result<(), DisplayHostError>;
    fn size(&self) -> DisplaySize;
}

pub trait ClockHost: Send + Sync {
    fn epoch_millis(&self) -> u64;
}

pub trait GuestOutputHost: Send + Sync {
    fn write_stdout(&self, bytes: &[u8]);
    fn write_stderr(&self, bytes: &[u8]);
}

pub trait ExitHost: Send + Sync {
    fn request_exit(&self);
}

pub trait VibrationHost: Send + Sync {
    fn vibrate(&self, duration_ms: u64, intensity: u8);
}

pub type HostFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuestFilesystemErrorCode {
    OperationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestFilesystemError {
    pub code: GuestFilesystemErrorCode,
    pub message: String,
}

impl GuestFilesystemError {
    #[must_use]
    pub fn operation_failed(message: impl Into<String>) -> Self {
        Self {
            code: GuestFilesystemErrorCode::OperationFailed,
            message: message.into(),
        }
    }
}

impl fmt::Display for GuestFilesystemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl Error for GuestFilesystemError {}

pub trait GuestFilesystemHost: Send + Sync {
    fn exists<'a>(&'a self, aid: &'a str, path: &'a str) -> HostFuture<'a, Result<bool, GuestFilesystemError>>;

    fn size<'a>(&'a self, aid: &'a str, path: &'a str) -> HostFuture<'a, Result<Option<usize>, GuestFilesystemError>>;

    fn read<'a>(
        &'a self,
        aid: &'a str,
        path: &'a str,
        offset: usize,
        count: usize,
        buf: &'a mut [u8],
    ) -> HostFuture<'a, Result<Option<usize>, GuestFilesystemError>>;

    fn write<'a>(
        &'a self,
        aid: &'a str,
        path: &'a str,
        offset: usize,
        data: &'a [u8],
    ) -> HostFuture<'a, Result<usize, GuestFilesystemError>>;

    fn truncate<'a>(
        &'a self,
        aid: &'a str,
        path: &'a str,
        len: usize,
    ) -> HostFuture<'a, Result<(), GuestFilesystemError>>;
}

pub type GuestDatabaseRecordId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuestDatabaseErrorCode {
    OperationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestDatabaseError {
    pub code: GuestDatabaseErrorCode,
    pub message: String,
}

impl GuestDatabaseError {
    #[must_use]
    pub fn operation_failed(message: impl Into<String>) -> Self {
        Self {
            code: GuestDatabaseErrorCode::OperationFailed,
            message: message.into(),
        }
    }
}

impl fmt::Display for GuestDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl Error for GuestDatabaseError {}

pub trait GuestDatabaseHost: Send + Sync {
    fn next_id<'a>(&'a self) -> HostFuture<'a, Result<GuestDatabaseRecordId, GuestDatabaseError>>;

    fn add<'a>(&'a mut self, data: &'a [u8]) -> HostFuture<'a, Result<GuestDatabaseRecordId, GuestDatabaseError>>;

    fn get<'a>(&'a self, id: GuestDatabaseRecordId) -> HostFuture<'a, Result<Option<Vec<u8>>, GuestDatabaseError>>;

    fn set<'a>(
        &'a mut self,
        id: GuestDatabaseRecordId,
        data: &'a [u8],
    ) -> HostFuture<'a, Result<bool, GuestDatabaseError>>;

    fn delete<'a>(&'a mut self, id: GuestDatabaseRecordId) -> HostFuture<'a, Result<bool, GuestDatabaseError>>;

    fn record_ids<'a>(&'a self) -> HostFuture<'a, Result<Vec<GuestDatabaseRecordId>, GuestDatabaseError>>;
}

pub trait GuestDatabaseRepositoryHost: Send + Sync {
    fn open<'a>(
        &'a self,
        name: &'a str,
        app_id: &'a str,
    ) -> HostFuture<'a, Result<Box<dyn GuestDatabaseHost>, GuestDatabaseError>>;

    fn exists<'a>(&'a self, name: &'a str, app_id: &'a str) -> HostFuture<'a, Result<bool, GuestDatabaseError>>;

    fn delete<'a>(&'a self, name: &'a str, app_id: &'a str) -> HostFuture<'a, Result<bool, GuestDatabaseError>>;

    fn usage<'a>(&'a self, app_id: &'a str) -> HostFuture<'a, Result<u64, GuestDatabaseError>>;
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
    use std::sync::Mutex;

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

    struct SyntheticClock(u64);

    impl ClockHost for SyntheticClock {
        fn epoch_millis(&self) -> u64 {
            self.0
        }
    }

    #[derive(Default)]
    struct SyntheticOutput {
        stdout: Mutex<Vec<u8>>,
        stderr: Mutex<Vec<u8>>,
    }

    impl GuestOutputHost for SyntheticOutput {
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
    struct SyntheticExit {
        requests: Mutex<u32>,
    }

    impl ExitHost for SyntheticExit {
        fn request_exit(&self) {
            let mut requests = self.requests.lock().expect("exit mutex poisoned");
            *requests += 1;
        }
    }

    #[derive(Default)]
    struct SyntheticVibration {
        last: Mutex<Option<(u64, u8)>>,
    }

    impl VibrationHost for SyntheticVibration {
        fn vibrate(&self, duration_ms: u64, intensity: u8) {
            *self.last.lock().expect("vibration mutex poisoned") = Some((duration_ms, intensity));
        }
    }

    struct SyntheticFilesystem;

    impl GuestFilesystemHost for SyntheticFilesystem {
        fn exists<'a>(&'a self, aid: &'a str, path: &'a str) -> HostFuture<'a, Result<bool, GuestFilesystemError>> {
            Box::pin(async move {
                assert_eq!(aid, "app-1");
                assert_eq!(path, "save/state.bin");
                Ok(true)
            })
        }

        fn size<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
        ) -> HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async move {
                assert_eq!(aid, "app-1");
                assert_eq!(path, "save/state.bin");
                Ok(Some(4))
            })
        }

        fn read<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
            offset: usize,
            count: usize,
            buf: &'a mut [u8],
        ) -> HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async move {
                assert_eq!(aid, "app-1");
                assert_eq!(path, "save/state.bin");
                assert_eq!(offset, 1);
                assert_eq!(count, 2);
                buf[..2].copy_from_slice(&[0x22, 0x33]);
                Ok(Some(2))
            })
        }

        fn write<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
            offset: usize,
            data: &'a [u8],
        ) -> HostFuture<'a, Result<usize, GuestFilesystemError>> {
            Box::pin(async move {
                assert_eq!(aid, "app-1");
                assert_eq!(path, "save/state.bin");
                assert_eq!(offset, 4);
                assert_eq!(data, &[0x44, 0x55]);
                Ok(data.len())
            })
        }

        fn truncate<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
            len: usize,
        ) -> HostFuture<'a, Result<(), GuestFilesystemError>> {
            Box::pin(async move {
                assert_eq!(aid, "app-1");
                assert_eq!(path, "save/state.bin");
                assert_eq!(len, 3);
                Ok(())
            })
        }
    }

    fn poll_ready<F: Future>(future: F) -> F::Output {
        use std::task::{Context, Poll, Waker};

        let mut future = Box::pin(future);
        let mut context = Context::from_waker(Waker::noop());

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[derive(Default)]
    struct SyntheticDatabase {
        records: Vec<(GuestDatabaseRecordId, Vec<u8>)>,
        next: GuestDatabaseRecordId,
    }

    impl SyntheticDatabase {
        fn with_seed() -> Self {
            Self {
                records: vec![(7, vec![1, 2, 3])],
                next: 8,
            }
        }
    }

    impl GuestDatabaseHost for SyntheticDatabase {
        fn next_id<'a>(&'a self) -> HostFuture<'a, Result<GuestDatabaseRecordId, GuestDatabaseError>> {
            Box::pin(async move { Ok(self.next) })
        }

        fn add<'a>(&'a mut self, data: &'a [u8]) -> HostFuture<'a, Result<GuestDatabaseRecordId, GuestDatabaseError>> {
            Box::pin(async move {
                let id = self.next;
                self.next += 1;
                self.records.push((id, data.to_vec()));
                Ok(id)
            })
        }

        fn get<'a>(&'a self, id: GuestDatabaseRecordId) -> HostFuture<'a, Result<Option<Vec<u8>>, GuestDatabaseError>> {
            Box::pin(async move {
                Ok(self
                    .records
                    .iter()
                    .find(|(record_id, _)| *record_id == id)
                    .map(|(_, data)| data.clone()))
            })
        }

        fn set<'a>(
            &'a mut self,
            id: GuestDatabaseRecordId,
            data: &'a [u8],
        ) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                let Some((_, stored)) = self.records.iter_mut().find(|(record_id, _)| *record_id == id) else {
                    return Ok(false);
                };

                *stored = data.to_vec();
                Ok(true)
            })
        }

        fn delete<'a>(&'a mut self, id: GuestDatabaseRecordId) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                let before = self.records.len();
                self.records.retain(|(record_id, _)| *record_id != id);
                Ok(self.records.len() != before)
            })
        }

        fn record_ids<'a>(&'a self) -> HostFuture<'a, Result<Vec<GuestDatabaseRecordId>, GuestDatabaseError>> {
            Box::pin(async move { Ok(self.records.iter().map(|(record_id, _)| *record_id).collect()) })
        }
    }

    #[derive(Default)]
    struct SyntheticDatabaseRepository;

    impl GuestDatabaseRepositoryHost for SyntheticDatabaseRepository {
        fn open<'a>(
            &'a self,
            name: &'a str,
            app_id: &'a str,
        ) -> HostFuture<'a, Result<Box<dyn GuestDatabaseHost>, GuestDatabaseError>> {
            Box::pin(async move {
                assert_eq!(name, "records");
                assert_eq!(app_id, "app-1");
                Ok(Box::new(SyntheticDatabase::with_seed()) as Box<dyn GuestDatabaseHost>)
            })
        }

        fn exists<'a>(&'a self, name: &'a str, app_id: &'a str) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                assert_eq!(name, "records");
                assert_eq!(app_id, "app-1");
                Ok(true)
            })
        }

        fn delete<'a>(&'a self, name: &'a str, app_id: &'a str) -> HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                assert_eq!(name, "records");
                assert_eq!(app_id, "app-1");
                Ok(true)
            })
        }

        fn usage<'a>(&'a self, app_id: &'a str) -> HostFuture<'a, Result<u64, GuestDatabaseError>> {
            Box::pin(async move {
                assert_eq!(app_id, "app-1");
                Ok(1_234)
            })
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
    fn display_size_calculates_rgba8_length() {
        let size = DisplaySize::new(240, 320);

        assert_eq!(size.pixel_count(), Some(76_800));
        assert_eq!(size.rgba8_byte_len(), Some(307_200));
    }

    #[test]
    fn rgba_frame_rejects_byte_length_mismatch() {
        let error = RgbaFrame::try_new(DisplaySize::new(2, 1), vec![0; 7])
            .expect_err("7 bytes must not represent a 2x1 RGBA8 frame");

        assert_eq!(error.code, FrameValidationErrorCode::ByteLengthMismatch);
        assert_eq!(error.expected_bytes, Some(8));
        assert_eq!(error.actual_bytes, 7);
    }

    #[test]
    fn rgba_frame_accepts_exact_byte_length() {
        let frame =
            RgbaFrame::try_new(DisplaySize::new(2, 1), vec![0; 8]).expect("8 bytes must represent a 2x1 RGBA8 frame");

        assert_eq!(frame.size, DisplaySize::new(2, 1));
        assert_eq!(frame.pixels.len(), 8);
    }

    #[test]
    fn display_host_error_keeps_stable_code_and_message() {
        let error = DisplayHostError::new(DisplayHostErrorCode::PresentFailed, "synthetic failure");

        assert_eq!(error.code, DisplayHostErrorCode::PresentFailed);
        assert_eq!(error.message, "synthetic failure");
        assert!(error.to_string().contains("PresentFailed"));
    }

    #[test]
    fn clock_host_preserves_full_epoch_millis() {
        let clock: &dyn ClockHost = &SyntheticClock(u64::MAX - 7);

        assert_eq!(clock.epoch_millis(), u64::MAX - 7);
    }

    #[test]
    fn guest_output_host_preserves_raw_bytes() {
        let output = SyntheticOutput::default();

        output.write_stdout(&[0x00, 0xFF, b'A']);
        output.write_stderr(&[0x80, b'B']);

        assert_eq!(
            *output.stdout.lock().expect("stdout mutex poisoned"),
            vec![0x00, 0xFF, b'A']
        );
        assert_eq!(*output.stderr.lock().expect("stderr mutex poisoned"), vec![0x80, b'B']);
    }

    #[test]
    fn exit_host_represents_guest_exit_request() {
        let exit = SyntheticExit::default();

        exit.request_exit();
        exit.request_exit();

        assert_eq!(*exit.requests.lock().expect("exit mutex poisoned"), 2);
    }

    #[test]
    fn vibration_host_preserves_duration_and_intensity() {
        let vibration = SyntheticVibration::default();

        vibration.vibrate(1_250, 200);

        assert_eq!(
            *vibration.last.lock().expect("vibration mutex poisoned"),
            Some((1_250, 200))
        );
    }

    #[test]
    fn guest_filesystem_error_keeps_stable_code_and_message() {
        let error = GuestFilesystemError::operation_failed("synthetic filesystem failure");

        assert_eq!(error.code, GuestFilesystemErrorCode::OperationFailed);
        assert_eq!(error.message, "synthetic filesystem failure");
        assert!(error.to_string().contains("OperationFailed"));
    }

    #[test]
    fn guest_filesystem_contract_preserves_aid_path_offsets_and_bytes() {
        let filesystem: &dyn GuestFilesystemHost = &SyntheticFilesystem;

        assert!(poll_ready(filesystem.exists("app-1", "save/state.bin")).expect("exists must succeed"));
        assert_eq!(
            poll_ready(filesystem.size("app-1", "save/state.bin")).expect("size must succeed"),
            Some(4)
        );

        let mut buffer = [0_u8; 4];
        assert_eq!(
            poll_ready(filesystem.read("app-1", "save/state.bin", 1, 2, &mut buffer)).expect("read must succeed"),
            Some(2)
        );
        assert_eq!(&buffer[..2], &[0x22, 0x33]);

        assert_eq!(
            poll_ready(filesystem.write("app-1", "save/state.bin", 4, &[0x44, 0x55])).expect("write must succeed"),
            2
        );

        poll_ready(filesystem.truncate("app-1", "save/state.bin", 3)).expect("truncate must succeed");
    }

    #[test]
    fn guest_database_record_id_is_exact_u32() {
        let record_id: GuestDatabaseRecordId = u32::MAX;

        assert_eq!(record_id, u32::MAX);
    }

    #[test]
    fn guest_database_error_keeps_stable_code_and_message() {
        let error = GuestDatabaseError::operation_failed("synthetic database failure");

        assert_eq!(error.code, GuestDatabaseErrorCode::OperationFailed);
        assert_eq!(error.message, "synthetic database failure");
        assert!(error.to_string().contains("OperationFailed"));
    }

    #[test]
    fn guest_database_contract_preserves_record_ids_and_bytes() {
        let mut database = SyntheticDatabase::with_seed();

        assert_eq!(poll_ready(database.next_id()).expect("next_id must succeed"), 8);
        assert_eq!(
            poll_ready(database.get(7)).expect("get must succeed"),
            Some(vec![1, 2, 3])
        );

        assert!(poll_ready(database.set(7, &[9, 8])).expect("set must succeed"));
        assert_eq!(
            poll_ready(database.get(7)).expect("get after set must succeed"),
            Some(vec![9, 8])
        );

        let added = poll_ready(database.add(&[4, 5])).expect("add must succeed");
        assert_eq!(added, 8);
        assert_eq!(
            poll_ready(database.record_ids()).expect("record_ids must succeed"),
            vec![7, 8]
        );

        assert!(poll_ready(database.delete(7)).expect("delete must succeed"));
        assert_eq!(
            poll_ready(database.record_ids()).expect("record_ids after delete must succeed"),
            vec![8]
        );
    }

    #[test]
    fn guest_database_repository_preserves_name_app_id_and_usage() {
        let repository: &dyn GuestDatabaseRepositoryHost = &SyntheticDatabaseRepository;

        assert!(poll_ready(repository.exists("records", "app-1")).expect("exists must succeed"));
        assert_eq!(
            poll_ready(repository.usage("app-1")).expect("usage must succeed"),
            1_234
        );

        let database = poll_ready(repository.open("records", "app-1")).expect("open must succeed");
        assert_eq!(
            poll_ready(database.get(7)).expect("opened database get must succeed"),
            Some(vec![1, 2, 3])
        );

        assert!(poll_ready(repository.delete("records", "app-1")).expect("delete must succeed"));
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
