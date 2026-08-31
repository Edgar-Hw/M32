use std::{fmt, sync::Arc};

use egui::{Context, ViewportId};
use egui_winit::State as EguiWinitState;
use m32_display::DisplayRenderer;
use m32_ui::{
    BootStatus, FirstPlayableShell, MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, REFERENCE_WINDOW_HEIGHT,
    REFERENCE_WINDOW_WIDTH, apply_theme,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const WINDOW_TITLE: &str = "M32 - Mobile Entertainment System";

#[derive(Debug)]
pub struct DesktopError(String);

impl fmt::Display for DesktopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for DesktopError {}

pub fn run() -> Result<(), DesktopError> {
    let event_loop = EventLoop::new().map_err(|error| DesktopError(format!("create native event loop: {error}")))?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = DesktopRuntime::new();

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
    fatal_error: Option<String>,
}

impl DesktopRuntime {
    fn new() -> Self {
        let context = Context::default();
        apply_theme(&context);

        Self {
            context,
            window: None,
            egui_state: None,
            renderer: None,
            shell: None,
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

        tracing::info!(
            target: "m32::display",
            event = "native_renderer_ready",
            adapter = %adapter_name,
            width = REFERENCE_WINDOW_WIDTH,
            height = REFERENCE_WINDOW_HEIGHT,
            "M32 native desktop renderer initialized"
        );

        self.shell = Some(FirstPlayableShell::new(BootStatus::bundle_a_ready()));
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

        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };

        let raw_input = match self.egui_state.as_mut() {
            Some(egui_state) => egui_state.take_egui_input(window.as_ref()),
            None => return,
        };

        let context = self.context.clone();
        let output = context.run_ui(raw_input, |ui| {
            if let Some(shell) = self.shell.as_mut() {
                shell.show(ui);
            }
        });

        let platform_output = match self.renderer.as_mut() {
            Some(renderer) => renderer.render(&self.context, &window, output),
            None => return,
        };

        if let Some(egui_state) = self.egui_state.as_mut() {
            egui_state.handle_platform_output_with_event_loop(window.as_ref(), event_loop, platform_output);
        }

        // First Playable will continuously present game frames. Keeping one
        // redraw loop here avoids inventing a second timing model in Bundle A.
        window.request_redraw();
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        tracing::error!(
            target: "m32::display",
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
