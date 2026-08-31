use std::{fmt, path::PathBuf, sync::Arc};

use egui::{ColorImage, Context, TextureHandle, TextureOptions, Vec2, ViewportId};
use egui_winit::State as EguiWinitState;
use m32_display::DisplayRenderer;
use m32_emulator_api::M32Key;
use m32_ui::{
    BootStatus, FirstPlayableShell, GameViewport, MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, REFERENCE_WINDOW_HEIGHT,
    REFERENCE_WINDOW_WIDTH, RuntimeDeckStatus, apply_theme,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::composition::{LocalLaunchRequest, PlayableRuntime};

const WINDOW_TITLE: &str = "M32 - Mobile Entertainment System";

#[derive(Debug)]
pub struct DesktopError(String);

impl fmt::Display for DesktopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for DesktopError {}

pub fn run(m32_root: PathBuf, launch: Option<LocalLaunchRequest>) -> Result<(), DesktopError> {
    let event_loop = EventLoop::new().map_err(|error| DesktopError(format!("create native event loop: {error}")))?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = DesktopRuntime::new(m32_root, launch);

    event_loop
        .run_app(&mut app)
        .map_err(|error| DesktopError(format!("run native event loop: {error}")))?;

    if let Some(error) = app.fatal_error.take() {
        return Err(DesktopError(error));
    }

    Ok(())
}

struct DesktopRuntime {
    context: Context,
    window: Option<Arc<Window>>,
    egui_state: Option<EguiWinitState>,
    renderer: Option<DisplayRenderer>,
    shell: Option<FirstPlayableShell>,
    playable: Option<PlayableRuntime>,
    launch: Option<LocalLaunchRequest>,
    m32_root: PathBuf,
    guest_texture: Option<TextureHandle>,
    guest_frame_revision: u64,
    guest_source_size: Option<Vec2>,
    fatal_error: Option<String>,
}

impl DesktopRuntime {
    fn new(m32_root: PathBuf, launch: Option<LocalLaunchRequest>) -> Self {
        let context = Context::default();
        apply_theme(&context);

        Self {
            context,
            window: None,
            egui_state: None,
            renderer: None,
            shell: None,
            playable: None,
            launch,
            m32_root,
            guest_texture: None,
            guest_frame_revision: 0,
            guest_source_size: None,
            fatal_error: None,
        }
    }

    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(
                REFERENCE_WINDOW_WIDTH as f64,
                REFERENCE_WINDOW_HEIGHT as f64,
            ))
            .with_min_inner_size(LogicalSize::new(MIN_WINDOW_WIDTH as f64, MIN_WINDOW_HEIGHT as f64))
            .with_visible(false);

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|error| format!("create M32 window: {error}"))?,
        );

        let renderer = DisplayRenderer::new(&self.context, Arc::clone(&window)).map_err(|error| error.to_string())?;

        let adapter_name = renderer.adapter_name().unwrap_or_else(|| "unknown".to_owned());

        let egui_state = EguiWinitState::new(
            self.context.clone(),
            ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            window.theme(),
            renderer.max_texture_side(),
        );

        let has_local_game = self.launch.is_some();
        if let Some(request) = self.launch.as_ref() {
            let playable = PlayableRuntime::launch_local(&self.m32_root, request).map_err(|error| error.to_string())?;
            tracing::info!(
                target: "m32::emulator",
                event = "local_jad_jar_launch_ready",
                jad = %request.jad_path.display(),
                jar = %request.jar_path.display(),
                "local JAD+JAR playable runtime created"
            );
            self.playable = Some(playable);
        }

        tracing::info!(
            target: "m32::display",
            event = "native_renderer_ready",
            adapter = %adapter_name,
            width = REFERENCE_WINDOW_WIDTH,
            height = REFERENCE_WINDOW_HEIGHT,
            "M32 native desktop renderer initialized"
        );

        self.shell = Some(FirstPlayableShell::new(BootStatus::first_playable(has_local_game)));
        self.egui_state = Some(egui_state);
        self.renderer = Some(renderer);
        self.window = Some(Arc::clone(&window));

        window.set_visible(true);
        window.request_redraw();

        Ok(())
    }

    fn recover_renderer_if_needed(&mut self, event_loop: &ActiveEventLoop) -> bool {
        let device_loss = self.renderer.as_ref().and_then(DisplayRenderer::take_device_loss);

        let Some(device_loss) = device_loss else {
            return true;
        };

        let Some(window) = self.window.as_ref().cloned() else {
            return false;
        };

        tracing::warn!(
            target: "m32::display",
            event = "gpu_device_lost",
            detail = %device_loss,
            "M32 GPU device was lost; recreating renderer"
        );

        match DisplayRenderer::new(&self.context, Arc::clone(&window)) {
            Ok(renderer) => {
                if let (Some(egui_state), Some(max_texture_side)) =
                    (self.egui_state.as_mut(), renderer.max_texture_side())
                {
                    egui_state.set_max_texture_side(max_texture_side);
                }
                self.renderer = Some(renderer);
                self.guest_texture = None;
                self.guest_frame_revision = 0;
                tracing::info!(
                    target: "m32::display",
                    event = "gpu_renderer_recovered",
                    "M32 GPU renderer recovered after device loss"
                );
                true
            }
            Err(error) => {
                self.fail(event_loop, format!("recreate renderer after device loss: {error}"));
                false
            }
        }
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if !self.recover_renderer_if_needed(event_loop) {
            return;
        }

        if let Some(playable) = self.playable.as_mut() {
            if let Err(error) = playable.pump() {
                self.fail(event_loop, format!("playable runtime tick failed: {error}"));
                return;
            }
            if playable.exit_requested() {
                event_loop.exit();
                return;
            }
        }

        self.refresh_guest_texture();

        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };

        let raw_input = match self.egui_state.as_mut() {
            Some(egui_state) => egui_state.take_egui_input(window.as_ref()),
            None => return,
        };

        let viewport = self.game_viewport();
        let runtime_status = RuntimeDeckStatus {
            game_loaded: self.playable.is_some(),
            input_live: self.playable.is_some(),
        };

        let context = self.context.clone();
        let output = context.run_ui(raw_input, |ui| {
            if let Some(shell) = self.shell.as_mut() {
                shell.show(ui, viewport, runtime_status);
            }
        });

        let platform_output = match self.renderer.as_mut() {
            Some(renderer) => renderer.render(&self.context, &window, output),
            None => return,
        };

        if let Some(egui_state) = self.egui_state.as_mut() {
            egui_state.handle_platform_output_with_event_loop(window.as_ref(), event_loop, platform_output);
        }

        window.request_redraw();
    }

    fn refresh_guest_texture(&mut self) {
        let Some(playable) = self.playable.as_ref() else {
            return;
        };
        let Some(snapshot) = playable.latest_frame_after(self.guest_frame_revision) else {
            return;
        };

        let width = snapshot.frame.size.width as usize;
        let height = snapshot.frame.size.height as usize;
        if width == 0 || height == 0 {
            return;
        }

        let image = ColorImage::from_rgba_unmultiplied([width, height], &snapshot.frame.pixels);

        if let Some(texture) = self.guest_texture.as_mut() {
            texture.set(image, TextureOptions::NEAREST);
        } else {
            self.guest_texture = Some(
                self.context
                    .load_texture("m32-guest-frame", image, TextureOptions::NEAREST),
            );
        }

        self.guest_frame_revision = snapshot.revision;
        self.guest_source_size = Some(Vec2::new(width as f32, height as f32));
    }

    fn game_viewport(&self) -> Option<GameViewport> {
        Some(GameViewport {
            texture_id: self.guest_texture.as_ref()?.id(),
            source_size: self.guest_source_size?,
        })
    }

    fn handle_guest_keyboard(&mut self, event: &winit::event::KeyEvent) {
        if event.repeat {
            return;
        }

        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let Some(key) = map_key_code(code) else {
            return;
        };
        let Some(playable) = self.playable.as_mut() else {
            return;
        };

        match event.state {
            ElementState::Pressed => {
                let _ = playable.key_down(key);
            }
            ElementState::Released => playable.key_up(key),
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        tracing::error!(
            target: "m32::lifecycle",
            event = "native_desktop_failed",
            error = %error,
            "M32 native desktop failed"
        );
        self.fatal_error = Some(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for DesktopRuntime {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        if let Err(error) = self.initialize_window(event_loop) {
            self.fail(event_loop, error);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        let response = self
            .egui_state
            .as_mut()
            .map(|state| state.on_window_event(window.as_ref(), &event));

        if response.is_some_and(|response| response.repaint) {
            window.request_redraw();
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut()) {
                    renderer.resize(window.inner_size());
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.handle_guest_keyboard(&event),
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn map_key_code(code: KeyCode) -> Option<M32Key> {
    Some(match code {
        KeyCode::ArrowUp => M32Key::Up,
        KeyCode::ArrowDown => M32Key::Down,
        KeyCode::ArrowLeft => M32Key::Left,
        KeyCode::ArrowRight => M32Key::Right,
        KeyCode::Enter | KeyCode::NumpadEnter => M32Key::Ok,
        KeyCode::KeyZ => M32Key::LeftSoft,
        KeyCode::KeyX => M32Key::RightSoft,
        KeyCode::Escape => M32Key::Clear,
        KeyCode::Digit0 | KeyCode::Numpad0 => M32Key::Num0,
        KeyCode::Digit1 | KeyCode::Numpad1 => M32Key::Num1,
        KeyCode::Digit2 | KeyCode::Numpad2 => M32Key::Num2,
        KeyCode::Digit3 | KeyCode::Numpad3 => M32Key::Num3,
        KeyCode::Digit4 | KeyCode::Numpad4 => M32Key::Num4,
        KeyCode::Digit5 | KeyCode::Numpad5 => M32Key::Num5,
        KeyCode::Digit6 | KeyCode::Numpad6 => M32Key::Num6,
        KeyCode::Digit7 | KeyCode::Numpad7 => M32Key::Num7,
        KeyCode::Digit8 | KeyCode::Numpad8 => M32Key::Num8,
        KeyCode::Digit9 | KeyCode::Numpad9 => M32Key::Num9,
        KeyCode::KeyA => M32Key::Star,
        KeyCode::KeyS => M32Key::Hash,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_mapping_matches_locked_first_playable_defaults() {
        assert_eq!(map_key_code(KeyCode::ArrowUp), Some(M32Key::Up));
        assert_eq!(map_key_code(KeyCode::Enter), Some(M32Key::Ok));
        assert_eq!(map_key_code(KeyCode::KeyZ), Some(M32Key::LeftSoft));
        assert_eq!(map_key_code(KeyCode::KeyX), Some(M32Key::RightSoft));
        assert_eq!(map_key_code(KeyCode::Escape), Some(M32Key::Clear));
        assert_eq!(map_key_code(KeyCode::Digit0), Some(M32Key::Num0));
        assert_eq!(map_key_code(KeyCode::Digit9), Some(M32Key::Num9));
        assert_eq!(map_key_code(KeyCode::KeyA), Some(M32Key::Star));
        assert_eq!(map_key_code(KeyCode::KeyS), Some(M32Key::Hash));
    }
}
