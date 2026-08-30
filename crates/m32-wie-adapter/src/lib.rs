//! Adapter boundary between M32 and the pinned WIE upstream.
//!
//! WIE backend types stay private to this crate. M32-facing callers consume contracts from
//! `m32-emulator-api`.

use std::{
    fmt::Display,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};

use m32_emulator_api::{
    BackendDescriptor, ClockHost, DisplayHost, DisplayHostError, DisplaySize, EmulatorBackend, EmulatorSession,
    EmulatorSessionCreateError, EmulatorSessionError, ExitHost, GuestAudioCommand, GuestAudioEventData, GuestAudioHost,
    GuestAudioHostError, GuestAudioSequence, GuestDatabaseError, GuestDatabaseHost, GuestDatabaseRepositoryHost,
    GuestFilesystemError, GuestFilesystemHost, GuestOutputHost, GuestTimedAudioEvent, HostServiceKind, RgbaFrame,
    SessionCreateErrorCode, SessionErrorCode, SessionState, VibrationHost,
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

pub struct WiePlatformHosts {
    pub display: Arc<dyn DisplayHost>,
    pub clock: Arc<dyn ClockHost>,
    pub database: Arc<dyn GuestDatabaseRepositoryHost>,
    pub filesystem: Arc<dyn GuestFilesystemHost>,
    pub audio: Arc<dyn GuestAudioHost>,
    pub output: Arc<dyn GuestOutputHost>,
    pub exit: Arc<dyn ExitHost>,
    pub vibration: Arc<dyn VibrationHost>,
}

pub struct WiePlatformAdapter {
    screen: WieScreenAdapter,
    database_repository: WieDatabaseRepositoryAdapter,
    filesystem: WieFilesystemAdapter,
    audio: Arc<dyn GuestAudioHost>,
    basic: WieBasicHostBridge,
}

impl WiePlatformAdapter {
    #[must_use]
    pub fn new(hosts: WiePlatformHosts) -> Self {
        Self {
            screen: WieScreenAdapter::new(hosts.display),
            database_repository: WieDatabaseRepositoryAdapter::new(hosts.database),
            filesystem: WieFilesystemAdapter::new(hosts.filesystem),
            audio: hosts.audio,
            basic: WieBasicHostBridge::new(hosts.clock, hosts.output, hosts.exit, hosts.vibration),
        }
    }
}

impl wie_backend::Platform for WiePlatformAdapter {
    fn screen(&self) -> &dyn wie_backend::Screen {
        &self.screen
    }

    fn now(&self) -> wie_backend::Instant {
        wie_backend::Instant::from_epoch_millis(self.basic.epoch_millis())
    }

    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        &self.database_repository
    }

    fn filesystem(&self) -> &dyn wie_backend::Filesystem {
        &self.filesystem
    }

    fn audio_sink(&self) -> Box<dyn wie_backend::AudioSink> {
        Box::new(WieAudioSinkAdapter::new(self.audio.clone()))
    }

    fn write_stdout(&self, buf: &[u8]) {
        self.basic.write_stdout(buf);
    }

    fn write_stderr(&self, buf: &[u8]) {
        self.basic.write_stderr(buf);
    }

    fn exit(&self) {
        self.basic.request_exit();
    }

    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        self.basic.vibrate(duration_ms, intensity);
    }
}

pub fn create_j2me_jar_session(
    hosts: WiePlatformHosts,
    jar_filename: &str,
    jar: Vec<u8>,
) -> Result<Box<dyn EmulatorSession>, EmulatorSessionCreateError> {
    let platform: Box<dyn wie_backend::Platform> = Box::new(WiePlatformAdapter::new(hosts));

    let emulator = wie_j2me::J2MEEmulator::from_jar(platform, jar_filename, jar).map_err(map_session_create_error)?;

    Ok(Box::new(WieSession {
        emulator: Box::new(emulator),
        state: SessionState::Ready,
    }))
}

pub fn create_j2me_jad_jar_session(
    hosts: WiePlatformHosts,
    jad: Vec<u8>,
    jar_filename: String,
    jar: Vec<u8>,
) -> Result<Box<dyn EmulatorSession>, EmulatorSessionCreateError> {
    let platform: Box<dyn wie_backend::Platform> = Box::new(WiePlatformAdapter::new(hosts));

    let emulator =
        wie_j2me::J2MEEmulator::from_jad_jar(platform, jad, jar_filename, jar).map_err(map_session_create_error)?;

    Ok(Box::new(WieSession {
        emulator: Box::new(emulator),
        state: SessionState::Ready,
    }))
}

fn map_session_create_error(error: impl Display) -> EmulatorSessionCreateError {
    EmulatorSessionCreateError::new(SessionCreateErrorCode::BackendLaunchFailed, error.to_string())
}

pub struct WieAudioSinkAdapter {
    host: Arc<dyn GuestAudioHost>,
}

impl WieAudioSinkAdapter {
    #[must_use]
    pub fn new(host: Arc<dyn GuestAudioHost>) -> Self {
        Self { host }
    }
}

impl wie_backend::AudioSink for WieAudioSinkAdapter {
    fn send(&self, command: wie_backend::AudioCommand) {
        let command = map_wie_audio_command(command);

        if let Err(error) = self.host.dispatch(command) {
            log_audio_host_error(&error);
        }
    }
}

fn map_wie_audio_command(command: wie_backend::AudioCommand) -> GuestAudioCommand {
    match command {
        wie_backend::AudioCommand::Play {
            handle,
            sequence,
            repeat,
        } => GuestAudioCommand::Play {
            handle,
            sequence: map_wie_audio_sequence(sequence.as_ref()),
            repeat,
        },
        wie_backend::AudioCommand::Stop { handle } => GuestAudioCommand::Stop { handle },
    }
}

fn map_wie_audio_sequence(sequence: &wie_backend::AudioSequence) -> GuestAudioSequence {
    GuestAudioSequence {
        duration: sequence.duration,
        events: sequence.events.iter().map(map_wie_timed_audio_event).collect(),
    }
}

fn map_wie_timed_audio_event(event: &wie_backend::TimedAudioEvent) -> GuestTimedAudioEvent {
    GuestTimedAudioEvent {
        time: event.time,
        data: match &event.data {
            wie_backend::AudioEventData::Midi(bytes) => GuestAudioEventData::Midi(bytes.clone()),
            wie_backend::AudioEventData::Wave {
                channels,
                sampling_rate,
                samples,
            } => GuestAudioEventData::Wave {
                channels: *channels,
                sampling_rate: *sampling_rate,
                samples: samples.clone(),
            },
        },
    }
}

fn log_audio_host_error(error: &GuestAudioHostError) {
    tracing::warn!(
        target: "m32::audio",
        event = "wie_audio_host_failed",
        error_code = ?error.code,
        "M32 guest audio host rejected a WIE audio command"
    );
}

pub struct WieDatabaseRepositoryAdapter {
    host: Arc<dyn GuestDatabaseRepositoryHost>,
}

impl WieDatabaseRepositoryAdapter {
    #[must_use]
    pub fn new(host: Arc<dyn GuestDatabaseRepositoryHost>) -> Self {
        Self { host }
    }
}

#[async_trait::async_trait]
impl wie_backend::DatabaseRepository for WieDatabaseRepositoryAdapter {
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn wie_backend::Database> {
        match self.host.open(name, app_id).await {
            Ok(database) => Box::new(WieDatabaseAdapter::new(database)),
            Err(error) => {
                log_database_host_error("open", &error);
                Box::new(UnavailableWieDatabase)
            }
        }
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        match self.host.exists(name, app_id).await {
            Ok(exists) => exists,
            Err(error) => {
                log_database_host_error("exists", &error);
                false
            }
        }
    }

    async fn delete(&self, name: &str, app_id: &str) -> bool {
        match self.host.delete(name, app_id).await {
            Ok(deleted) => deleted,
            Err(error) => {
                log_database_host_error("delete_repository", &error);
                false
            }
        }
    }

    async fn usage(&self, app_id: &str) -> u64 {
        match self.host.usage(app_id).await {
            Ok(usage) => usage,
            Err(error) => {
                log_database_host_error("usage", &error);
                0
            }
        }
    }
}

pub struct WieDatabaseAdapter {
    host: Box<dyn GuestDatabaseHost>,
}

impl WieDatabaseAdapter {
    #[must_use]
    pub fn new(host: Box<dyn GuestDatabaseHost>) -> Self {
        Self { host }
    }
}

#[async_trait::async_trait]
impl wie_backend::Database for WieDatabaseAdapter {
    async fn next_id(&self) -> wie_backend::RecordId {
        match self.host.next_id().await {
            Ok(id) => id,
            Err(error) => {
                log_database_host_error("next_id", &error);
                0
            }
        }
    }

    async fn add(&mut self, data: &[u8]) -> wie_backend::RecordId {
        match self.host.add(data).await {
            Ok(id) => id,
            Err(error) => {
                log_database_host_error("add", &error);
                0
            }
        }
    }

    async fn get(&self, id: wie_backend::RecordId) -> Option<Vec<u8>> {
        match self.host.get(id).await {
            Ok(data) => data,
            Err(error) => {
                log_database_host_error("get", &error);
                None
            }
        }
    }

    async fn set(&mut self, id: wie_backend::RecordId, data: &[u8]) -> bool {
        match self.host.set(id, data).await {
            Ok(updated) => updated,
            Err(error) => {
                log_database_host_error("set", &error);
                false
            }
        }
    }

    async fn delete(&mut self, id: wie_backend::RecordId) -> bool {
        match self.host.delete(id).await {
            Ok(deleted) => deleted,
            Err(error) => {
                log_database_host_error("delete_record", &error);
                false
            }
        }
    }

    async fn get_record_ids(&self) -> Vec<wie_backend::RecordId> {
        match self.host.record_ids().await {
            Ok(record_ids) => record_ids,
            Err(error) => {
                log_database_host_error("record_ids", &error);
                Vec::new()
            }
        }
    }
}

struct UnavailableWieDatabase;

#[async_trait::async_trait]
impl wie_backend::Database for UnavailableWieDatabase {
    async fn next_id(&self) -> wie_backend::RecordId {
        0
    }

    async fn add(&mut self, _data: &[u8]) -> wie_backend::RecordId {
        0
    }

    async fn get(&self, _id: wie_backend::RecordId) -> Option<Vec<u8>> {
        None
    }

    async fn set(&mut self, _id: wie_backend::RecordId, _data: &[u8]) -> bool {
        false
    }

    async fn delete(&mut self, _id: wie_backend::RecordId) -> bool {
        false
    }

    async fn get_record_ids(&self) -> Vec<wie_backend::RecordId> {
        Vec::new()
    }
}

fn log_database_host_error(operation: &'static str, error: &GuestDatabaseError) {
    tracing::warn!(
        target: "m32::storage",
        event = "wie_database_host_failed",
        operation,
        error_code = ?error.code,
        "M32 guest database host operation failed"
    );
}

pub struct WieFilesystemAdapter {
    host: Arc<dyn GuestFilesystemHost>,
}

impl WieFilesystemAdapter {
    #[must_use]
    pub fn new(host: Arc<dyn GuestFilesystemHost>) -> Self {
        Self { host }
    }
}

#[async_trait::async_trait]
impl wie_backend::Filesystem for WieFilesystemAdapter {
    async fn exists(&self, aid: &str, path: &str) -> bool {
        match self.host.exists(aid, path).await {
            Ok(exists) => exists,
            Err(error) => {
                log_filesystem_host_error("exists", &error);
                false
            }
        }
    }

    async fn size(&self, aid: &str, path: &str) -> Option<usize> {
        match self.host.size(aid, path).await {
            Ok(size) => size,
            Err(error) => {
                log_filesystem_host_error("size", &error);
                None
            }
        }
    }

    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        match self.host.read(aid, path, offset, count, buf).await {
            Ok(Some(read)) if read <= count && read <= buf.len() => Some(read),
            Ok(Some(read)) => {
                tracing::warn!(
                    target: "m32::storage",
                    event = "wie_filesystem_invalid_read_count",
                    returned_count = read,
                    requested_count = count,
                    buffer_len = buf.len(),
                    "M32 filesystem host returned an invalid WIE read count"
                );
                None
            }
            Ok(None) => None,
            Err(error) => {
                log_filesystem_host_error("read", &error);
                None
            }
        }
    }

    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        match self.host.write(aid, path, offset, data).await {
            Ok(written) if written == data.len() => written,
            Ok(written) => {
                tracing::warn!(
                    target: "m32::storage",
                    event = "wie_filesystem_invalid_write_count",
                    returned_count = written,
                    requested_count = data.len(),
                    "M32 filesystem host returned a partial/invalid WIE write count"
                );
                0
            }
            Err(error) => {
                log_filesystem_host_error("write", &error);
                0
            }
        }
    }

    async fn truncate(&self, aid: &str, path: &str, len: usize) {
        if let Err(error) = self.host.truncate(aid, path, len).await {
            log_filesystem_host_error("truncate", &error);
        }
    }
}

fn log_filesystem_host_error(operation: &'static str, error: &GuestFilesystemError) {
    tracing::warn!(
        target: "m32::storage",
        event = "wie_filesystem_host_failed",
        operation,
        error_code = ?error.code,
        "M32 guest filesystem host operation failed"
    );
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
        match catch_unwind(AssertUnwindSafe(|| self.emulator.tick())) {
            Ok(Ok(())) => {
                self.state = SessionState::Running;
                Ok(())
            }
            Ok(Err(error)) => {
                self.state = SessionState::Faulted;
                Err(map_tick_error(error))
            }
            Err(_) => {
                self.state = SessionState::Faulted;
                tracing::error!(
                    target: "m32::emulator",
                    event = "wie_tick_panicked",
                    "Pinned WIE backend panicked during tick"
                );
                Err(map_tick_panic())
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

fn map_tick_panic() -> EmulatorSessionError {
    EmulatorSessionError::new(
        SessionErrorCode::BackendTickFailed,
        "pinned WIE backend panicked during tick",
    )
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

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordedWrite {
        aid: String,
        path: String,
        offset: usize,
        data: Vec<u8>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordedTruncate {
        aid: String,
        path: String,
        len: usize,
    }

    #[derive(Default)]
    struct RecordingFilesystemHost {
        calls: Mutex<Vec<String>>,
        write: Mutex<Option<RecordedWrite>>,
        truncate: Mutex<Option<RecordedTruncate>>,
    }

    impl GuestFilesystemHost for RecordingFilesystemHost {
        fn exists<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestFilesystemError>> {
            Box::pin(async move {
                self.calls
                    .lock()
                    .expect("filesystem calls mutex poisoned")
                    .push(format!("exists:{aid}:{path}"));
                Ok(path == "save/state.bin")
            })
        }

        fn size<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async move {
                self.calls
                    .lock()
                    .expect("filesystem calls mutex poisoned")
                    .push(format!("size:{aid}:{path}"));
                Ok(Some(6))
            })
        }

        fn read<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
            offset: usize,
            count: usize,
            buf: &'a mut [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async move {
                self.calls
                    .lock()
                    .expect("filesystem calls mutex poisoned")
                    .push(format!("read:{aid}:{path}:{offset}:{count}"));

                let source = b"abcdef";
                if offset >= source.len() {
                    return Ok(Some(0));
                }

                let read = count.min(source.len() - offset);
                buf[..read].copy_from_slice(&source[offset..offset + read]);
                Ok(Some(read))
            })
        }

        fn write<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
            offset: usize,
            data: &'a [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<usize, GuestFilesystemError>> {
            Box::pin(async move {
                *self.write.lock().expect("filesystem write mutex poisoned") = Some(RecordedWrite {
                    aid: aid.to_owned(),
                    path: path.to_owned(),
                    offset,
                    data: data.to_vec(),
                });
                Ok(data.len())
            })
        }

        fn truncate<'a>(
            &'a self,
            aid: &'a str,
            path: &'a str,
            len: usize,
        ) -> m32_emulator_api::HostFuture<'a, Result<(), GuestFilesystemError>> {
            Box::pin(async move {
                *self.truncate.lock().expect("filesystem truncate mutex poisoned") = Some(RecordedTruncate {
                    aid: aid.to_owned(),
                    path: path.to_owned(),
                    len,
                });
                Ok(())
            })
        }
    }

    struct FailingFilesystemHost;

    impl GuestFilesystemHost for FailingFilesystemHost {
        fn exists<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestFilesystemError>> {
            Box::pin(async { Err(GuestFilesystemError::operation_failed("exists failure")) })
        }

        fn size<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async { Err(GuestFilesystemError::operation_failed("size failure")) })
        }

        fn read<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
            _offset: usize,
            _count: usize,
            _buf: &'a mut [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async { Err(GuestFilesystemError::operation_failed("read failure")) })
        }

        fn write<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
            _offset: usize,
            _data: &'a [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<usize, GuestFilesystemError>> {
            Box::pin(async { Err(GuestFilesystemError::operation_failed("write failure")) })
        }

        fn truncate<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
            _len: usize,
        ) -> m32_emulator_api::HostFuture<'a, Result<(), GuestFilesystemError>> {
            Box::pin(async { Err(GuestFilesystemError::operation_failed("truncate failure")) })
        }
    }

    struct InvalidCountFilesystemHost;

    impl GuestFilesystemHost for InvalidCountFilesystemHost {
        fn exists<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestFilesystemError>> {
            Box::pin(async { Ok(false) })
        }

        fn size<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async { Ok(None) })
        }

        fn read<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
            _offset: usize,
            count: usize,
            _buf: &'a mut [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<Option<usize>, GuestFilesystemError>> {
            Box::pin(async move { Ok(Some(count + 1)) })
        }

        fn write<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
            _offset: usize,
            data: &'a [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<usize, GuestFilesystemError>> {
            Box::pin(async move { Ok(data.len().saturating_sub(1)) })
        }

        fn truncate<'a>(
            &'a self,
            _aid: &'a str,
            _path: &'a str,
            _len: usize,
        ) -> m32_emulator_api::HostFuture<'a, Result<(), GuestFilesystemError>> {
            Box::pin(async { Ok(()) })
        }
    }

    fn block_on_ready<F: std::future::Future>(future: F) -> F::Output {
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

    #[test]
    fn wie_filesystem_adapter_implements_pinned_filesystem_contract() {
        fn assert_filesystem<T: wie_backend::Filesystem>() {}

        assert_filesystem::<WieFilesystemAdapter>();
    }

    #[test]
    fn wie_filesystem_exists_and_size_preserve_aid_and_path() {
        let host = Arc::new(RecordingFilesystemHost::default());
        let filesystem = WieFilesystemAdapter::new(host.clone());

        assert!(block_on_ready(wie_backend::Filesystem::exists(
            &filesystem,
            "game.aid",
            "save/state.bin"
        )));
        assert_eq!(
            block_on_ready(wie_backend::Filesystem::size(&filesystem, "game.aid", "save/state.bin")),
            Some(6)
        );

        assert_eq!(
            *host.calls.lock().expect("filesystem calls mutex poisoned"),
            vec!["exists:game.aid:save/state.bin", "size:game.aid:save/state.bin"]
        );
    }

    #[test]
    fn wie_filesystem_read_preserves_offset_count_and_buffer_result() {
        let host = Arc::new(RecordingFilesystemHost::default());
        let filesystem = WieFilesystemAdapter::new(host);

        let mut buffer = [0_u8; 4];
        let read = block_on_ready(wie_backend::Filesystem::read(
            &filesystem,
            "game.aid",
            "save/state.bin",
            2,
            3,
            &mut buffer,
        ));

        assert_eq!(read, Some(3));
        assert_eq!(&buffer[..3], b"cde");
    }

    #[test]
    fn wie_filesystem_write_and_truncate_preserve_request_values() {
        let host = Arc::new(RecordingFilesystemHost::default());
        let filesystem = WieFilesystemAdapter::new(host.clone());

        assert_eq!(
            block_on_ready(wie_backend::Filesystem::write(
                &filesystem,
                "game.aid",
                "save/state.bin",
                7,
                &[1, 2, 3],
            )),
            3
        );
        block_on_ready(wie_backend::Filesystem::truncate(
            &filesystem,
            "game.aid",
            "save/state.bin",
            9,
        ));

        assert_eq!(
            *host.write.lock().expect("filesystem write mutex poisoned"),
            Some(RecordedWrite {
                aid: "game.aid".to_owned(),
                path: "save/state.bin".to_owned(),
                offset: 7,
                data: vec![1, 2, 3],
            })
        );
        assert_eq!(
            *host.truncate.lock().expect("filesystem truncate mutex poisoned"),
            Some(RecordedTruncate {
                aid: "game.aid".to_owned(),
                path: "save/state.bin".to_owned(),
                len: 9,
            })
        );
    }

    #[test]
    fn wie_filesystem_host_errors_map_to_wie_fallback_values() {
        let filesystem = WieFilesystemAdapter::new(Arc::new(FailingFilesystemHost));
        let mut buffer = [0_u8; 4];

        assert!(!block_on_ready(wie_backend::Filesystem::exists(
            &filesystem,
            "aid",
            "file"
        )));
        assert_eq!(
            block_on_ready(wie_backend::Filesystem::size(&filesystem, "aid", "file")),
            None
        );
        assert_eq!(
            block_on_ready(wie_backend::Filesystem::read(
                &filesystem,
                "aid",
                "file",
                0,
                4,
                &mut buffer
            )),
            None
        );
        assert_eq!(
            block_on_ready(wie_backend::Filesystem::write(&filesystem, "aid", "file", 0, &[1, 2])),
            0
        );

        block_on_ready(wie_backend::Filesystem::truncate(&filesystem, "aid", "file", 0));
    }

    #[test]
    fn wie_filesystem_rejects_invalid_host_read_and_write_counts() {
        let filesystem = WieFilesystemAdapter::new(Arc::new(InvalidCountFilesystemHost));
        let mut buffer = [0_u8; 2];

        assert_eq!(
            block_on_ready(wie_backend::Filesystem::read(
                &filesystem,
                "aid",
                "file",
                0,
                2,
                &mut buffer
            )),
            None
        );
        assert_eq!(
            block_on_ready(wie_backend::Filesystem::write(&filesystem, "aid", "file", 0, &[1, 2])),
            0
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordedDatabaseOpen {
        name: String,
        app_id: String,
    }

    #[derive(Default)]
    struct RecordingDatabase {
        records: Vec<(u32, Vec<u8>)>,
        next_id: u32,
    }

    impl RecordingDatabase {
        fn seeded() -> Self {
            Self {
                records: vec![(3, vec![1, 2, 3])],
                next_id: 4,
            }
        }
    }

    impl GuestDatabaseHost for RecordingDatabase {
        fn next_id<'a>(&'a self) -> m32_emulator_api::HostFuture<'a, Result<u32, GuestDatabaseError>> {
            Box::pin(async move { Ok(self.next_id) })
        }

        fn add<'a>(&'a mut self, data: &'a [u8]) -> m32_emulator_api::HostFuture<'a, Result<u32, GuestDatabaseError>> {
            Box::pin(async move {
                let id = self.next_id;
                self.next_id += 1;
                self.records.push((id, data.to_vec()));
                Ok(id)
            })
        }

        fn get<'a>(&'a self, id: u32) -> m32_emulator_api::HostFuture<'a, Result<Option<Vec<u8>>, GuestDatabaseError>> {
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
            id: u32,
            data: &'a [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                let Some((_, stored)) = self.records.iter_mut().find(|(record_id, _)| *record_id == id) else {
                    return Ok(false);
                };

                *stored = data.to_vec();
                Ok(true)
            })
        }

        fn delete<'a>(&'a mut self, id: u32) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                let before = self.records.len();
                self.records.retain(|(record_id, _)| *record_id != id);
                Ok(before != self.records.len())
            })
        }

        fn record_ids<'a>(&'a self) -> m32_emulator_api::HostFuture<'a, Result<Vec<u32>, GuestDatabaseError>> {
            Box::pin(async move { Ok(self.records.iter().map(|(record_id, _)| *record_id).collect()) })
        }
    }

    #[derive(Default)]
    struct RecordingDatabaseRepository {
        last_open: Mutex<Option<RecordedDatabaseOpen>>,
        last_exists: Mutex<Option<RecordedDatabaseOpen>>,
        last_delete: Mutex<Option<RecordedDatabaseOpen>>,
        last_usage_app_id: Mutex<Option<String>>,
    }

    impl GuestDatabaseRepositoryHost for RecordingDatabaseRepository {
        fn open<'a>(
            &'a self,
            name: &'a str,
            app_id: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<Box<dyn GuestDatabaseHost>, GuestDatabaseError>> {
            Box::pin(async move {
                *self.last_open.lock().expect("database open mutex poisoned") = Some(RecordedDatabaseOpen {
                    name: name.to_owned(),
                    app_id: app_id.to_owned(),
                });

                Ok(Box::new(RecordingDatabase::seeded()) as Box<dyn GuestDatabaseHost>)
            })
        }

        fn exists<'a>(
            &'a self,
            name: &'a str,
            app_id: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                *self.last_exists.lock().expect("database exists mutex poisoned") = Some(RecordedDatabaseOpen {
                    name: name.to_owned(),
                    app_id: app_id.to_owned(),
                });
                Ok(true)
            })
        }

        fn delete<'a>(
            &'a self,
            name: &'a str,
            app_id: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async move {
                *self.last_delete.lock().expect("database delete mutex poisoned") = Some(RecordedDatabaseOpen {
                    name: name.to_owned(),
                    app_id: app_id.to_owned(),
                });
                Ok(true)
            })
        }

        fn usage<'a>(&'a self, app_id: &'a str) -> m32_emulator_api::HostFuture<'a, Result<u64, GuestDatabaseError>> {
            Box::pin(async move {
                *self.last_usage_app_id.lock().expect("database usage mutex poisoned") = Some(app_id.to_owned());
                Ok(456)
            })
        }
    }

    struct FailingDatabase;

    impl GuestDatabaseHost for FailingDatabase {
        fn next_id<'a>(&'a self) -> m32_emulator_api::HostFuture<'a, Result<u32, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("next_id failure")) })
        }

        fn add<'a>(&'a mut self, _data: &'a [u8]) -> m32_emulator_api::HostFuture<'a, Result<u32, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("add failure")) })
        }

        fn get<'a>(
            &'a self,
            _id: u32,
        ) -> m32_emulator_api::HostFuture<'a, Result<Option<Vec<u8>>, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("get failure")) })
        }

        fn set<'a>(
            &'a mut self,
            _id: u32,
            _data: &'a [u8],
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("set failure")) })
        }

        fn delete<'a>(&'a mut self, _id: u32) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("delete failure")) })
        }

        fn record_ids<'a>(&'a self) -> m32_emulator_api::HostFuture<'a, Result<Vec<u32>, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("record_ids failure")) })
        }
    }

    struct FailingDatabaseRepository;

    impl GuestDatabaseRepositoryHost for FailingDatabaseRepository {
        fn open<'a>(
            &'a self,
            _name: &'a str,
            _app_id: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<Box<dyn GuestDatabaseHost>, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("open failure")) })
        }

        fn exists<'a>(
            &'a self,
            _name: &'a str,
            _app_id: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("exists failure")) })
        }

        fn delete<'a>(
            &'a self,
            _name: &'a str,
            _app_id: &'a str,
        ) -> m32_emulator_api::HostFuture<'a, Result<bool, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("repository delete failure")) })
        }

        fn usage<'a>(&'a self, _app_id: &'a str) -> m32_emulator_api::HostFuture<'a, Result<u64, GuestDatabaseError>> {
            Box::pin(async { Err(GuestDatabaseError::operation_failed("usage failure")) })
        }
    }

    #[test]
    fn wie_database_adapters_implement_pinned_database_contracts() {
        fn assert_database<T: wie_backend::Database>() {}
        fn assert_repository<T: wie_backend::DatabaseRepository>() {}

        assert_database::<WieDatabaseAdapter>();
        assert_repository::<WieDatabaseRepositoryAdapter>();
    }

    #[test]
    fn wie_database_repository_preserves_name_app_id_and_usage() {
        let host = Arc::new(RecordingDatabaseRepository::default());
        let repository = WieDatabaseRepositoryAdapter::new(host.clone());

        assert!(block_on_ready(wie_backend::DatabaseRepository::exists(
            &repository,
            "records",
            "game.app"
        )));
        assert_eq!(
            block_on_ready(wie_backend::DatabaseRepository::usage(&repository, "game.app")),
            456
        );
        assert!(block_on_ready(wie_backend::DatabaseRepository::delete(
            &repository,
            "records",
            "game.app"
        )));

        assert_eq!(
            *host.last_exists.lock().expect("database exists mutex poisoned"),
            Some(RecordedDatabaseOpen {
                name: "records".to_owned(),
                app_id: "game.app".to_owned(),
            })
        );
        assert_eq!(
            *host.last_delete.lock().expect("database delete mutex poisoned"),
            Some(RecordedDatabaseOpen {
                name: "records".to_owned(),
                app_id: "game.app".to_owned(),
            })
        );
        assert_eq!(
            *host.last_usage_app_id.lock().expect("database usage mutex poisoned"),
            Some("game.app".to_owned())
        );
    }

    #[test]
    fn wie_database_open_returns_working_record_bridge() {
        let host = Arc::new(RecordingDatabaseRepository::default());
        let repository = WieDatabaseRepositoryAdapter::new(host.clone());

        let mut database = block_on_ready(wie_backend::DatabaseRepository::open(
            &repository,
            "records",
            "game.app",
        ));

        assert_eq!(
            *host.last_open.lock().expect("database open mutex poisoned"),
            Some(RecordedDatabaseOpen {
                name: "records".to_owned(),
                app_id: "game.app".to_owned(),
            })
        );

        assert_eq!(block_on_ready(database.next_id()), 4);
        assert_eq!(block_on_ready(database.get(3)), Some(vec![1, 2, 3]));

        assert!(block_on_ready(database.set(3, &[7, 8])));
        assert_eq!(block_on_ready(database.get(3)), Some(vec![7, 8]));

        assert_eq!(block_on_ready(database.add(&[9])), 4);
        assert_eq!(block_on_ready(database.get_record_ids()), vec![3, 4]);

        assert!(block_on_ready(database.delete(3)));
        assert_eq!(block_on_ready(database.get_record_ids()), vec![4]);
    }

    #[test]
    fn wie_database_repository_errors_map_to_safe_fallbacks() {
        let repository = WieDatabaseRepositoryAdapter::new(Arc::new(FailingDatabaseRepository));

        assert!(!block_on_ready(wie_backend::DatabaseRepository::exists(
            &repository,
            "records",
            "game.app"
        )));
        assert!(!block_on_ready(wie_backend::DatabaseRepository::delete(
            &repository,
            "records",
            "game.app"
        )));
        assert_eq!(
            block_on_ready(wie_backend::DatabaseRepository::usage(&repository, "game.app")),
            0
        );

        let mut database = block_on_ready(wie_backend::DatabaseRepository::open(
            &repository,
            "records",
            "game.app",
        ));
        assert_eq!(block_on_ready(database.next_id()), 0);
        assert_eq!(block_on_ready(database.add(&[1, 2])), 0);
        assert_eq!(block_on_ready(database.get(1)), None);
        assert!(!block_on_ready(database.set(1, &[3])));
        assert!(!block_on_ready(database.delete(1)));
        assert!(block_on_ready(database.get_record_ids()).is_empty());
    }

    #[test]
    fn wie_database_record_errors_map_to_safe_fallbacks() {
        let mut database = WieDatabaseAdapter::new(Box::new(FailingDatabase));

        assert_eq!(block_on_ready(wie_backend::Database::next_id(&database)), 0);
        assert_eq!(block_on_ready(wie_backend::Database::add(&mut database, &[1])), 0);
        assert_eq!(block_on_ready(wie_backend::Database::get(&database, 7)), None);
        assert!(!block_on_ready(wie_backend::Database::set(&mut database, 7, &[2])));
        assert!(!block_on_ready(wie_backend::Database::delete(&mut database, 7)));
        assert!(block_on_ready(wie_backend::Database::get_record_ids(&database)).is_empty());
    }

    #[derive(Default)]
    struct RecordingAudioHost {
        commands: Mutex<Vec<GuestAudioCommand>>,
    }

    impl GuestAudioHost for RecordingAudioHost {
        fn dispatch(&self, command: GuestAudioCommand) -> Result<(), GuestAudioHostError> {
            self.commands
                .lock()
                .expect("audio commands mutex poisoned")
                .push(command);
            Ok(())
        }
    }

    struct FailingAudioHost;

    impl GuestAudioHost for FailingAudioHost {
        fn dispatch(&self, _command: GuestAudioCommand) -> Result<(), GuestAudioHostError> {
            Err(GuestAudioHostError::dispatch_failed("synthetic audio dispatch failure"))
        }
    }

    #[test]
    fn wie_audio_sink_adapter_implements_pinned_audio_sink_contract() {
        fn assert_audio_sink<T: wie_backend::AudioSink>() {}

        assert_audio_sink::<WieAudioSinkAdapter>();
    }

    #[test]
    fn wie_audio_play_maps_handle_repeat_duration_and_midi_bytes() {
        let host = Arc::new(RecordingAudioHost::default());
        let sink = WieAudioSinkAdapter::new(host.clone());

        wie_backend::AudioSink::send(
            &sink,
            wie_backend::AudioCommand::Play {
                handle: 9,
                sequence: Arc::new(wie_backend::AudioSequence {
                    duration: 1_500,
                    events: vec![wie_backend::TimedAudioEvent {
                        time: 25,
                        data: wie_backend::AudioEventData::Midi(vec![0x90, 64, 100]),
                    }],
                }),
                repeat: true,
            },
        );

        assert_eq!(
            *host.commands.lock().expect("audio commands mutex poisoned"),
            vec![GuestAudioCommand::Play {
                handle: 9,
                sequence: GuestAudioSequence {
                    duration: 1_500,
                    events: vec![GuestTimedAudioEvent {
                        time: 25,
                        data: GuestAudioEventData::Midi(vec![0x90, 64, 100]),
                    }],
                },
                repeat: true,
            }]
        );
    }

    #[test]
    fn wie_audio_wave_maps_channels_sampling_rate_and_i16_samples() {
        let host = Arc::new(RecordingAudioHost::default());
        let sink = WieAudioSinkAdapter::new(host.clone());

        wie_backend::AudioSink::send(
            &sink,
            wie_backend::AudioCommand::Play {
                handle: 10,
                sequence: Arc::new(wie_backend::AudioSequence {
                    duration: 300,
                    events: vec![wie_backend::TimedAudioEvent {
                        time: 5,
                        data: wie_backend::AudioEventData::Wave {
                            channels: 2,
                            sampling_rate: 22_050,
                            samples: vec![-100, 0, 100, 200],
                        },
                    }],
                }),
                repeat: false,
            },
        );

        assert_eq!(
            *host.commands.lock().expect("audio commands mutex poisoned"),
            vec![GuestAudioCommand::Play {
                handle: 10,
                sequence: GuestAudioSequence {
                    duration: 300,
                    events: vec![GuestTimedAudioEvent {
                        time: 5,
                        data: GuestAudioEventData::Wave {
                            channels: 2,
                            sampling_rate: 22_050,
                            samples: vec![-100, 0, 100, 200],
                        },
                    }],
                },
                repeat: false,
            }]
        );
    }

    #[test]
    fn wie_audio_stop_maps_exact_handle() {
        let host = Arc::new(RecordingAudioHost::default());
        let sink = WieAudioSinkAdapter::new(host.clone());

        wie_backend::AudioSink::send(&sink, wie_backend::AudioCommand::Stop { handle: 77 });

        assert_eq!(
            *host.commands.lock().expect("audio commands mutex poisoned"),
            vec![GuestAudioCommand::Stop { handle: 77 }]
        );
    }

    #[test]
    fn wie_audio_host_failure_is_non_panicking() {
        let sink = WieAudioSinkAdapter::new(Arc::new(FailingAudioHost));

        wie_backend::AudioSink::send(&sink, wie_backend::AudioCommand::Stop { handle: 1 });
    }

    struct RecordingPlatformFixture {
        platform: WiePlatformAdapter,
        output: Arc<RecordingOutput>,
        exit: Arc<RecordingExit>,
        vibration: Arc<RecordingVibration>,
        filesystem: Arc<RecordingFilesystemHost>,
        database: Arc<RecordingDatabaseRepository>,
        audio: Arc<RecordingAudioHost>,
    }

    fn recording_platform() -> RecordingPlatformFixture {
        let display = Arc::new(RecordingDisplayHost::default());
        display
            .resize(DisplaySize::new(240, 320))
            .expect("synthetic display resize must succeed");

        let output = Arc::new(RecordingOutput::default());
        let exit = Arc::new(RecordingExit::default());
        let vibration = Arc::new(RecordingVibration::default());
        let filesystem = Arc::new(RecordingFilesystemHost::default());
        let database = Arc::new(RecordingDatabaseRepository::default());
        let audio = Arc::new(RecordingAudioHost::default());

        let platform = WiePlatformAdapter::new(WiePlatformHosts {
            display: display.clone(),
            clock: Arc::new(FixedClock(1_725_123_456_789)),
            database: database.clone(),
            filesystem: filesystem.clone(),
            audio: audio.clone(),
            output: output.clone(),
            exit: exit.clone(),
            vibration: vibration.clone(),
        });

        RecordingPlatformFixture {
            platform,
            output,
            exit,
            vibration,
            filesystem,
            database,
            audio,
        }
    }

    #[test]
    fn wie_platform_adapter_implements_pinned_platform_contract() {
        fn assert_platform<T: wie_backend::Platform>() {}

        assert_platform::<WiePlatformAdapter>();
    }

    #[test]
    fn wie_platform_delegates_screen_clock_database_and_filesystem() {
        let fixture = recording_platform();

        let platform_ref: &dyn wie_backend::Platform = &fixture.platform;

        assert_eq!(platform_ref.screen().width(), 240);
        assert_eq!(platform_ref.screen().height(), 320);
        assert_eq!(platform_ref.now().raw(), 1_725_123_456_789);

        assert!(block_on_ready(wie_backend::DatabaseRepository::exists(
            platform_ref.database_repository(),
            "records",
            "game.app"
        )));
        assert!(block_on_ready(wie_backend::Filesystem::exists(
            platform_ref.filesystem(),
            "game.aid",
            "save/state.bin"
        )));

        assert_eq!(
            *fixture
                .database
                .last_exists
                .lock()
                .expect("database exists mutex poisoned"),
            Some(RecordedDatabaseOpen {
                name: "records".to_owned(),
                app_id: "game.app".to_owned(),
            })
        );
        assert_eq!(
            *fixture
                .filesystem
                .calls
                .lock()
                .expect("filesystem calls mutex poisoned"),
            vec!["exists:game.aid:save/state.bin"]
        );
    }

    #[test]
    fn wie_platform_delegates_output_exit_and_vibration() {
        let fixture = recording_platform();

        let platform_ref: &dyn wie_backend::Platform = &fixture.platform;

        platform_ref.write_stdout(&[0x00, 0xFF, b'O']);
        platform_ref.write_stderr(&[0x80, b'E']);
        platform_ref.exit();
        platform_ref.vibrate(650, 144);

        assert_eq!(
            *fixture.output.stdout.lock().expect("stdout mutex poisoned"),
            vec![0x00, 0xFF, b'O']
        );
        assert_eq!(
            *fixture.output.stderr.lock().expect("stderr mutex poisoned"),
            vec![0x80, b'E']
        );
        assert_eq!(*fixture.exit.count.lock().expect("exit mutex poisoned"), 1);
        assert_eq!(
            *fixture.vibration.last.lock().expect("vibration mutex poisoned"),
            Some((650, 144))
        );
    }

    #[test]
    fn wie_platform_creates_audio_sink_backed_by_shared_m32_audio_host() {
        let fixture = recording_platform();

        let sink_a = wie_backend::Platform::audio_sink(&fixture.platform);
        let sink_b = wie_backend::Platform::audio_sink(&fixture.platform);

        sink_a.send(wie_backend::AudioCommand::Stop { handle: 11 });
        sink_b.send(wie_backend::AudioCommand::Stop { handle: 12 });

        assert_eq!(
            *fixture.audio.commands.lock().expect("audio commands mutex poisoned"),
            vec![
                GuestAudioCommand::Stop { handle: 11 },
                GuestAudioCommand::Stop { handle: 12 },
            ]
        );
    }

    fn recording_platform_hosts() -> WiePlatformHosts {
        WiePlatformHosts {
            display: Arc::new(RecordingDisplayHost::default()),
            clock: Arc::new(FixedClock(1_725_123_456_789)),
            database: Arc::new(RecordingDatabaseRepository::default()),
            filesystem: Arc::new(RecordingFilesystemHost::default()),
            audio: Arc::new(RecordingAudioHost::default()),
            output: Arc::new(RecordingOutput::default()),
            exit: Arc::new(RecordingExit::default()),
            vibration: Arc::new(RecordingVibration::default()),
        }
    }

    #[test]
    fn pinned_j2me_emulator_implements_wie_emulator_contract() {
        fn assert_emulator<T: wie_backend::Emulator>() {}

        assert_emulator::<wie_j2me::J2MEEmulator>();
    }

    #[test]
    fn j2me_jar_factory_constructs_ready_m32_session() {
        let session = create_j2me_jar_session(recording_platform_hosts(), "synthetic-empty.jar", Vec::new())
            .expect("pinned J2ME constructor must accept ownership of platform and JAR bytes");

        assert_eq!(session.backend(), wie_backend_descriptor());
        assert_eq!(session.state(), SessionState::Ready);
    }

    const FIRST_FRAME_LAUNCH_JAD: &[u8] = b"\
MIDlet-Name: M32 First Frame Smoke\r\n\
MIDlet-Version: 1.0.0\r\n\
MIDlet-Vendor: M32\r\n\
MIDlet-1: M32 First Frame,,m32.FirstFrameMidlet\r\n";

    #[test]
    fn j2me_jad_jar_factory_constructs_ready_m32_session() {
        let session = create_j2me_jad_jar_session(
            recording_platform_hosts(),
            FIRST_FRAME_LAUNCH_JAD.to_vec(),
            "m32-first-frame.jar".to_owned(),
            Vec::new(),
        )
        .expect("pinned JAD+JAR constructor must create a Ready M32 session");

        assert_eq!(session.backend(), wie_backend_descriptor());
        assert_eq!(session.state(), SessionState::Ready);
    }

    #[test]
    fn j2me_jad_jar_factory_uses_explicit_launch_descriptor_path() {
        let session = create_j2me_jad_jar_session(
            recording_platform_hosts(),
            FIRST_FRAME_LAUNCH_JAD.to_vec(),
            "no-manifest-required.jar".to_owned(),
            Vec::new(),
        )
        .expect("JAD+JAR factory must not require a JAR manifest during construction");

        assert_eq!(session.state(), SessionState::Ready);
    }

    const FIRST_FRAME_BOOT_JAD: &[u8] = include_bytes!("../test-fixtures/j2me-first-frame-boot.jad");
    const FIRST_FRAME_BOOT_JAR: &[u8] = include_bytes!("../test-fixtures/j2me-first-frame-boot.jar");

    fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|window| window == needle)
    }

    #[test]
    fn first_frame_boot_fixture_has_expected_container_and_class_identity() {
        assert!(FIRST_FRAME_BOOT_JAR.starts_with(b"PK\x03\x04"));
        assert!(contains_bytes(FIRST_FRAME_BOOT_JAR, b"m32/FirstFrameMidlet.class"));
        assert!(contains_bytes(FIRST_FRAME_BOOT_JAR, b"m32/FirstFrameMidlet"));
        assert!(contains_bytes(
            FIRST_FRAME_BOOT_JAR,
            &[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x34]
        ));
        assert!(contains_bytes(
            FIRST_FRAME_BOOT_JAD,
            b"MIDlet-1: M32 First Frame,,m32.FirstFrameMidlet"
        ));
    }

    #[test]
    fn first_frame_boot_fixture_constructs_ready_j2me_session() {
        let session = create_j2me_jad_jar_session(
            recording_platform_hosts(),
            FIRST_FRAME_BOOT_JAD.to_vec(),
            "j2me-first-frame-boot.jar".to_owned(),
            FIRST_FRAME_BOOT_JAR.to_vec(),
        )
        .expect("deterministic boot fixture must construct a Ready J2ME session");

        assert_eq!(session.backend(), wie_backend_descriptor());
        assert_eq!(session.state(), SessionState::Ready);
    }

    #[test]
    fn session_create_error_maps_to_stable_m32_code() {
        let error = map_session_create_error("synthetic constructor failure");

        assert_eq!(error.code, SessionCreateErrorCode::BackendLaunchFailed);
        assert_eq!(error.message, "synthetic constructor failure");
    }

    const CORE_SMOKE_MISSING_MAIN_JAR: &[u8] = include_bytes!("../test-fixtures/j2me-core-smoke-missing-main.jar");
    const CORE_SMOKE_MAX_TICKS: usize = 512;

    #[test]
    fn j2me_core_smoke_ticks_real_runtime_to_stable_fault_boundary() {
        let mut session = create_j2me_jar_session(
            recording_platform_hosts(),
            "j2me-core-smoke-missing-main.jar",
            CORE_SMOKE_MISSING_MAIN_JAR.to_vec(),
        )
        .expect("synthetic JAR constructor must create a Ready WIE J2ME session");

        assert_eq!(session.state(), SessionState::Ready);

        for _ in 0..CORE_SMOKE_MAX_TICKS {
            match session.tick() {
                Ok(()) => continue,
                Err(error) => {
                    assert_eq!(error.code, SessionErrorCode::BackendTickFailed);
                    assert_eq!(session.state(), SessionState::Faulted);
                    assert!(!error.message.is_empty());
                    return;
                }
            }
        }

        panic!(
            "synthetic missing-main JAR did not reach the expected fault boundary within {CORE_SMOKE_MAX_TICKS} ticks"
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

    struct PanickingWieEmulator;

    impl wie_backend::Emulator for PanickingWieEmulator {
        fn handle_event(&mut self, _event: wie_backend::Event) {}

        fn tick(&mut self) -> wie_util::Result<()> {
            panic!("synthetic upstream WIE panic");
        }
    }

    #[test]
    fn wie_tick_panic_maps_to_faulted_stable_m32_error() {
        let mut session = WieSession {
            emulator: Box::new(PanickingWieEmulator),
            state: SessionState::Ready,
        };

        let error = session
            .tick()
            .expect_err("upstream panic must be contained by the M32 adapter");

        assert_eq!(error.code, SessionErrorCode::BackendTickFailed);
        assert_eq!(error.message, "pinned WIE backend panicked during tick");
        assert_eq!(session.state(), SessionState::Faulted);
    }

    #[test]
    fn wie_tick_error_maps_to_stable_m32_error_code() {
        let error = map_tick_error("synthetic WIE tick failure");

        assert_eq!(error.code, SessionErrorCode::BackendTickFailed);
        assert_eq!(error.message, "synthetic WIE tick failure");
    }
}
