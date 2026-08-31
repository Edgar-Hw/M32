use std::{
    ffi::OsString,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use m32_audio::RealtimeGuestAudioHost;
use m32_emulator_api::{
    ClockHost, DisplayHost, DisplayHostError, DisplaySize, EmulatorSession, EmulatorSessionError, ExitHost,
    GuestAudioHost, GuestDatabaseRepositoryHost, GuestFilesystemHost, GuestOutputHost, M32Key, RgbaFrame,
    VibrationHost,
};
use m32_input::{GuestInputController, KeyDownOutcome};
use m32_storage::PersistentGuestStorage;
use m32_wie_adapter::{WiePlatformHosts, create_j2me_jad_jar_session};

const TICKS_PER_PUMP: usize = 4;
const DEFAULT_GUEST_WIDTH: u32 = 176;
const DEFAULT_GUEST_HEIGHT: u32 = 220;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalLaunchRequest {
    pub jad_path: PathBuf,
    pub jar_path: PathBuf,
}

impl LocalLaunchRequest {
    pub fn from_args<I>(args: I) -> Result<Option<Self>, LocalLaunchArgError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut args = args.into_iter();
        let _program = args.next();

        let mut jad_path = None;
        let mut jar_path = None;

        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--jad" => {
                    let value = args.next().ok_or(LocalLaunchArgError::MissingValue("--jad"))?;
                    jad_path = Some(PathBuf::from(value));
                }
                "--jar" => {
                    let value = args.next().ok_or(LocalLaunchArgError::MissingValue("--jar"))?;
                    jar_path = Some(PathBuf::from(value));
                }
                other => return Err(LocalLaunchArgError::UnknownArgument(other.to_owned())),
            }
        }

        match (jad_path, jar_path) {
            (None, None) => Ok(None),
            (Some(jad_path), Some(jar_path)) => Ok(Some(Self { jad_path, jar_path })),
            (Some(_), None) => Err(LocalLaunchArgError::MissingPair("--jar")),
            (None, Some(_)) => Err(LocalLaunchArgError::MissingPair("--jad")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalLaunchArgError {
    MissingValue(&'static str),
    MissingPair(&'static str),
    UnknownArgument(String),
}

impl fmt::Display for LocalLaunchArgError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValue(flag) => write!(formatter, "missing value after {flag}"),
            Self::MissingPair(flag) => write!(formatter, "local JAD+JAR launch also requires {flag}"),
            Self::UnknownArgument(argument) => write!(
                formatter,
                "unknown M32 argument '{argument}'; expected --jad <file.jad> --jar <file.jar>"
            ),
        }
    }
}

impl std::error::Error for LocalLaunchArgError {}

#[derive(Debug)]
pub enum PlayableLaunchError {
    JadExtension(PathBuf),
    JarExtension(PathBuf),
    JadRead { path: PathBuf, source: std::io::Error },
    JarRead { path: PathBuf, source: std::io::Error },
    JarFilename(PathBuf),
    Storage(String),
    Session(String),
}

impl fmt::Display for PlayableLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JadExtension(path) => write!(formatter, "JAD input must end in .jad: {}", path.display()),
            Self::JarExtension(path) => write!(formatter, "JAR input must end in .jar: {}", path.display()),
            Self::JadRead { path, source } => write!(formatter, "cannot read local JAD '{}': {source}", path.display()),
            Self::JarRead { path, source } => write!(formatter, "cannot read local JAR '{}': {source}", path.display()),
            Self::JarFilename(path) => write!(formatter, "local JAR has no usable filename: {}", path.display()),
            Self::Storage(message) => write!(formatter, "open M32 persistent guest storage: {message}"),
            Self::Session(message) => write!(formatter, "create WIE JAD+JAR session: {message}"),
        }
    }
}

impl std::error::Error for PlayableLaunchError {}

#[derive(Debug, Clone)]
pub struct LiveFrameSnapshot {
    pub revision: u64,
    pub frame: RgbaFrame,
}

#[derive(Debug)]
struct LiveDisplayState {
    size: DisplaySize,
    pending_redraws: u64,
    revision: u64,
    latest: Option<RgbaFrame>,
}

pub struct LiveDisplayHost {
    state: Mutex<LiveDisplayState>,
}

impl LiveDisplayHost {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(LiveDisplayState {
                size: DisplaySize::new(DEFAULT_GUEST_WIDTH, DEFAULT_GUEST_HEIGHT),
                pending_redraws: 0,
                revision: 0,
                latest: None,
            }),
        }
    }

    fn take_redraw_requests(&self) -> u64 {
        let Ok(mut state) = self.state.lock() else {
            return 0;
        };
        let requests = state.pending_redraws;
        state.pending_redraws = 0;
        requests
    }

    pub fn latest_after(&self, revision: u64) -> Option<LiveFrameSnapshot> {
        let state = self.state.lock().ok()?;
        if state.revision <= revision {
            return None;
        }

        Some(LiveFrameSnapshot {
            revision: state.revision,
            frame: state.latest.clone()?,
        })
    }

    #[cfg(test)]
    fn revision(&self) -> u64 {
        self.state.lock().map(|state| state.revision).unwrap_or(0)
    }
}

impl Default for LiveDisplayHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayHost for LiveDisplayHost {
    fn resize(&self, size: DisplaySize) -> Result<(), DisplayHostError> {
        if let Ok(mut state) = self.state.lock() {
            state.size = size;
        }
        Ok(())
    }

    fn request_redraw(&self) -> Result<(), DisplayHostError> {
        if let Ok(mut state) = self.state.lock() {
            state.pending_redraws = state.pending_redraws.saturating_add(1);
        }
        Ok(())
    }

    fn present_rgba8(&self, frame: RgbaFrame) -> Result<(), DisplayHostError> {
        if let Ok(mut state) = self.state.lock() {
            state.size = frame.size;
            state.revision = state.revision.saturating_add(1);
            state.latest = Some(frame);
        }
        Ok(())
    }

    fn size(&self) -> DisplaySize {
        self.state
            .lock()
            .map(|state| state.size)
            .unwrap_or(DisplaySize::new(DEFAULT_GUEST_WIDTH, DEFAULT_GUEST_HEIGHT))
    }
}

#[derive(Default)]
struct SystemClockHost;

impl ClockHost for SystemClockHost {
    fn epoch_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
            .unwrap_or(0)
    }
}

#[derive(Default)]
pub struct DesktopOutputHost {
    stdout: Mutex<Vec<u8>>,
    stderr: Mutex<Vec<u8>>,
}

impl DesktopOutputHost {
    #[cfg(test)]
    fn stdout_snapshot(&self) -> Vec<u8> {
        self.stdout.lock().map(|bytes| bytes.clone()).unwrap_or_default()
    }
}

impl GuestOutputHost for DesktopOutputHost {
    fn write_stdout(&self, bytes: &[u8]) {
        if let Ok(mut output) = self.stdout.lock() {
            output.extend_from_slice(bytes);
        }
        tracing::debug!(
            target: "m32::guest",
            event = "guest_stdout",
            bytes = bytes.len(),
            "guest stdout bytes received"
        );
    }

    fn write_stderr(&self, bytes: &[u8]) {
        if let Ok(mut output) = self.stderr.lock() {
            output.extend_from_slice(bytes);
        }
        tracing::warn!(
            target: "m32::guest",
            event = "guest_stderr",
            bytes = bytes.len(),
            "guest stderr bytes received"
        );
    }
}

#[derive(Default)]
struct DesktopExitHost {
    requested: AtomicBool,
}

impl ExitHost for DesktopExitHost {
    fn request_exit(&self) {
        self.requested.store(true, Ordering::Release);
    }
}

#[derive(Default)]
struct DesktopVibrationHost;

impl VibrationHost for DesktopVibrationHost {
    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        tracing::debug!(
            target: "m32::guest",
            event = "guest_vibration",
            duration_ms,
            intensity,
            "guest vibration request received"
        );
    }
}

pub struct PlayableRuntime {
    session: Box<dyn EmulatorSession>,
    display: Arc<LiveDisplayHost>,
    _output: Arc<DesktopOutputHost>,
    exit: Arc<DesktopExitHost>,
    input: GuestInputController,
    started_at: Instant,
    _audio: Arc<RealtimeGuestAudioHost>,
    _storage: PersistentGuestStorage,
}

impl PlayableRuntime {
    pub fn launch_local(m32_root: &Path, request: &LocalLaunchRequest) -> Result<Self, PlayableLaunchError> {
        validate_extension(&request.jad_path, "jad")
            .map_err(|()| PlayableLaunchError::JadExtension(request.jad_path.clone()))?;
        validate_extension(&request.jar_path, "jar")
            .map_err(|()| PlayableLaunchError::JarExtension(request.jar_path.clone()))?;

        let jad = fs::read(&request.jad_path).map_err(|source| PlayableLaunchError::JadRead {
            path: request.jad_path.clone(),
            source,
        })?;
        let jar = fs::read(&request.jar_path).map_err(|source| PlayableLaunchError::JarRead {
            path: request.jar_path.clone(),
            source,
        })?;
        let jar_filename = request
            .jar_path
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_owned)
            .ok_or_else(|| PlayableLaunchError::JarFilename(request.jar_path.clone()))?;

        let storage =
            PersistentGuestStorage::open(m32_root).map_err(|error| PlayableLaunchError::Storage(error.to_string()))?;

        let display = Arc::new(LiveDisplayHost::new());
        let output = Arc::new(DesktopOutputHost::default());
        let exit = Arc::new(DesktopExitHost::default());
        let audio = Arc::new(RealtimeGuestAudioHost::new());

        let database: Arc<dyn GuestDatabaseRepositoryHost> = Arc::new(storage.database_repository());
        let filesystem: Arc<dyn GuestFilesystemHost> = Arc::new(storage.filesystem());
        let audio_host: Arc<dyn GuestAudioHost> = audio.clone();

        let hosts = WiePlatformHosts {
            display: display.clone(),
            clock: Arc::new(SystemClockHost),
            database,
            filesystem,
            audio: audio_host,
            output: output.clone(),
            exit: exit.clone(),
            vibration: Arc::new(DesktopVibrationHost),
        };

        let session = create_j2me_jad_jar_session(hosts, jad, jar_filename, jar)
            .map_err(|error| PlayableLaunchError::Session(error.to_string()))?;

        Ok(Self {
            session,
            display,
            _output: output,
            exit,
            input: GuestInputController::new(),
            started_at: Instant::now(),
            _audio: audio,
            _storage: storage,
        })
    }

    pub fn pump(&mut self) -> Result<(), EmulatorSessionError> {
        let now_ms = self.elapsed_ms();
        for event in self.input.repeats_due(now_ms) {
            self.session.handle_input(event);
        }

        for _ in 0..TICKS_PER_PUMP {
            self.session.tick()?;
            self.forward_redraw_requests();
        }

        Ok(())
    }

    pub fn key_down(&mut self, key: M32Key) -> KeyDownOutcome {
        let (outcome, event) = self.input.key_down(key, self.elapsed_ms());
        if let Some(event) = event {
            self.session.handle_input(event);
        }
        outcome
    }

    pub fn key_up(&mut self, key: M32Key) {
        if let Some(event) = self.input.key_up(key) {
            self.session.handle_input(event);
        }
    }

    pub fn latest_frame_after(&self, revision: u64) -> Option<LiveFrameSnapshot> {
        self.display.latest_after(revision)
    }

    pub fn exit_requested(&self) -> bool {
        self.exit.requested.load(Ordering::Acquire)
    }

    fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
    }

    fn forward_redraw_requests(&mut self) {
        for _ in 0..self.display.take_redraw_requests() {
            self.session.handle_redraw();
        }
    }

    #[cfg(test)]
    fn output(&self) -> &Arc<DesktopOutputHost> {
        &self._output
    }
}

fn validate_extension(path: &Path, expected: &str) -> Result<(), ()> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case(expected) => Ok(()),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        thread,
        time::Duration,
    };

    use m32_emulator_api::{DisplayHost, DisplaySize};

    use super::*;

    const INPUT_JAD: &[u8] =
        include_bytes!("../../../crates/m32-wie-adapter/test-fixtures/j2me-input-key-observer.jad");
    const INPUT_JAR: &[u8] =
        include_bytes!("../../../crates/m32-wie-adapter/test-fixtures/j2me-input-key-observer.jar");
    const PAINT_JAD: &[u8] = include_bytes!("../../../crates/m32-wie-adapter/test-fixtures/j2me-first-frame-paint.jad");
    const PAINT_JAR: &[u8] = include_bytes!("../../../crates/m32-wie-adapter/test-fixtures/j2me-first-frame-paint.jar");

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(label: &str) -> Self {
            let id = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("m32-first-playable-{label}-{}-{id}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("test root must be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn fixture(&self, stem: &str, jad: &[u8], jar: &[u8]) -> LocalLaunchRequest {
            let jad_path = self.0.join(format!("{stem}.jad"));
            let jar_path = self.0.join(format!("{stem}.jar"));
            fs::write(&jad_path, jad).expect("fixture JAD write");
            fs::write(&jar_path, jar).expect("fixture JAR write");
            LocalLaunchRequest { jad_path, jar_path }
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn local_launch_args_require_explicit_jad_and_jar_pair() {
        let args = [
            OsString::from("m32"),
            OsString::from("--jad"),
            OsString::from("game.jad"),
        ];
        assert_eq!(
            LocalLaunchRequest::from_args(args),
            Err(LocalLaunchArgError::MissingPair("--jar"))
        );
    }

    #[test]
    fn local_launch_reports_missing_jad_clearly() {
        let root = TestRoot::new("missing-jad");
        let jar_path = root.path().join("game.jar");
        fs::write(&jar_path, PAINT_JAR).unwrap();

        let error = match PlayableRuntime::launch_local(
            root.path(),
            &LocalLaunchRequest {
                jad_path: root.path().join("missing.jad"),
                jar_path,
            },
        ) {
            Ok(_) => panic!("missing JAD must fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("cannot read local JAD"));
        assert!(error.to_string().contains("missing.jad"));
    }

    #[test]
    fn live_display_host_keeps_latest_frame_not_first_frame_only() {
        let host = LiveDisplayHost::new();
        let size = DisplaySize::new(1, 1);

        host.present_rgba8(RgbaFrame::try_new(size, vec![1, 2, 3, 255]).unwrap())
            .unwrap();
        let first = host.latest_after(0).unwrap();
        assert_eq!(first.revision, 1);
        assert_eq!(first.frame.pixels, vec![1, 2, 3, 255]);

        host.present_rgba8(RgbaFrame::try_new(size, vec![9, 8, 7, 255]).unwrap())
            .unwrap();
        let second = host.latest_after(first.revision).unwrap();
        assert_eq!(second.revision, 2);
        assert_eq!(second.frame.pixels, vec![9, 8, 7, 255]);
        assert_eq!(host.revision(), 2);
    }

    #[test]
    fn composed_local_fixture_reaches_real_guest_frame() {
        let root = TestRoot::new("frame");
        let request = root.fixture("paint", PAINT_JAD, PAINT_JAR);
        let mut runtime = PlayableRuntime::launch_local(root.path(), &request).expect("fixture launch");

        let revision = 0;
        for _ in 0..512 {
            runtime.pump().expect("fixture pump");
            if let Some(snapshot) = runtime.latest_frame_after(revision) {
                assert_eq!(snapshot.frame.size, DisplaySize::new(176, 220));
                assert_eq!(snapshot.frame.pixels.len(), 154_880);
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }

        panic!("composed local JAD+JAR fixture did not reach a guest frame");
    }

    #[test]
    fn composed_input_pump_reaches_real_canvas_key_pressed_callback() {
        let root = TestRoot::new("input");
        let request = root.fixture("input", INPUT_JAD, INPUT_JAR);
        let mut runtime = PlayableRuntime::launch_local(root.path(), &request).expect("fixture launch");

        pump_until_stdout(&mut runtime, b"M32_KEY_CANVAS_READY;");
        assert_eq!(runtime.key_down(M32Key::Up), KeyDownOutcome::Accepted);
        pump_until_stdout(&mut runtime, b"M32_KEY_PRESSED:141;");
        runtime.key_up(M32Key::Up);
        pump_until_stdout(&mut runtime, b"M32_KEY_RELEASED:141;");
    }

    fn pump_until_stdout(runtime: &mut PlayableRuntime, sentinel: &[u8]) {
        for _ in 0..512 {
            runtime.pump().expect("fixture pump");
            if contains_bytes(&runtime.output().stdout_snapshot(), sentinel) {
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
        panic!(
            "guest stdout sentinel not observed: {}",
            String::from_utf8_lossy(sentinel)
        );
    }

    fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|window| window == needle)
    }
}
