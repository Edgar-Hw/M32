//! M32 native GPU presentation boundary.
//!
//! Bundle A owns only the native egui/wgpu presentation surface. Guest-frame
//! texture upload is intentionally deferred to the later First Playable tasks.

use std::{
    fmt,
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use egui::{Context, FullOutput, PlatformOutput, ViewportId};
use egui_wgpu::{RendererOptions, WgpuConfiguration, winit::Painter};
use winit::{dpi::PhysicalSize, window::Window};

/// Locked M32 application background, `BG0 #0E1114`.
pub const CLEAR_COLOR: [f32; 4] = [14.0 / 255.0, 17.0 / 255.0, 20.0 / 255.0, 1.0];

#[derive(Debug)]
pub struct DisplayError {
    operation: &'static str,
    detail: String,
}

impl DisplayError {
    fn new(operation: &'static str, detail: impl Into<String>) -> Self {
        Self {
            operation,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for DisplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.detail)
    }
}

impl std::error::Error for DisplayError {}

/// Owns the native wgpu surface/device/queue renderer used by the M32 shell.
///
/// `egui_wgpu::winit::Painter` owns the actual wgpu surface, device and queue.
/// M32 keeps this wrapper as the stable display boundary so the desktop crate
/// never manipulates raw wgpu state directly.
pub struct DisplayRenderer {
    painter: Painter,
    device_loss: Arc<Mutex<Option<String>>>,
}

impl DisplayRenderer {
    pub fn new(context: &Context, window: Arc<Window>) -> Result<Self, DisplayError> {
        let mut painter = pollster::block_on(Painter::new(
            context.clone(),
            WgpuConfiguration::default(),
            false,
            RendererOptions::default(),
        ));

        pollster::block_on(painter.set_window(ViewportId::ROOT, Some(window)))
            .map_err(|error| DisplayError::new("initialize wgpu window surface", error.to_string()))?;

        let device_loss = Arc::new(Mutex::new(None));

        let render_state = painter.render_state().ok_or_else(|| {
            DisplayError::new("initialize wgpu render state", "render state missing after set_window")
        })?;

        let device_loss_for_callback = Arc::clone(&device_loss);
        render_state
            .device
            .set_device_lost_callback(move |reason: wgpu::DeviceLostReason, message: String| {
                let detail = format!("{reason:?}: {message}");
                if let Ok(mut slot) = device_loss_for_callback.lock() {
                    *slot = Some(detail);
                }
            });

        Ok(Self { painter, device_loss })
    }

    pub fn max_texture_side(&self) -> Option<usize> {
        self.painter.max_texture_side()
    }

    pub fn adapter_name(&self) -> Option<String> {
        self.painter
            .render_state()
            .map(|render_state| render_state.adapter.get_info().name)
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };

        self.painter.on_window_resized(ViewportId::ROOT, width, height);
    }

    /// Consume the most recent device-loss report, if any.
    ///
    /// The desktop composition layer uses this signal to recreate the renderer
    /// while keeping emulator ownership outside this crate.
    pub fn take_device_loss(&self) -> Option<String> {
        self.device_loss.lock().ok().and_then(|mut slot| slot.take())
    }

    /// Render a complete egui frame and return the platform output for winit.
    pub fn render(&mut self, context: &Context, window: &Arc<Window>, output: FullOutput) -> PlatformOutput {
        let FullOutput {
            platform_output,
            mut textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = output;

        let clipped_primitives = context.tessellate(shapes, pixels_per_point);

        self.painter.paint_and_update_textures(
            ViewportId::ROOT,
            pixels_per_point,
            CLEAR_COLOR,
            &clipped_primitives,
            &mut textures_delta,
            Vec::new(),
            window,
        );

        platform_output
    }
}

impl Drop for DisplayRenderer {
    fn drop(&mut self) {
        self.painter.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::CLEAR_COLOR;

    #[test]
    fn clear_color_matches_locked_bg0() {
        let expected = [14.0 / 255.0, 17.0 / 255.0, 20.0 / 255.0, 1.0];
        assert_eq!(CLEAR_COLOR, expected);
    }
}
