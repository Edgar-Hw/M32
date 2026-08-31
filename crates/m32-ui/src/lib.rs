//! M32 First Playable shell UI.
//!
//! The shell stays backend-agnostic. The desktop composition layer supplies an
//! optional egui texture representing the latest M32 RGBA8 guest frame.

use std::time::{Duration, Instant};

use egui::{Align, Align2, Color32, Context, FontId, Frame, Layout, Rect, RichText, Sense, Stroke, TextureId, Vec2};

pub const BOOT_DURATION_MS: u64 = 1_000;

pub const REFERENCE_WINDOW_WIDTH: f32 = 1_440.0;
pub const REFERENCE_WINDOW_HEIGHT: f32 = 900.0;
pub const MIN_WINDOW_WIDTH: f32 = 1_180.0;
pub const MIN_WINDOW_HEIGHT: f32 = 700.0;

pub const PLAY_REFERENCE_CONTENT_WIDTH: f32 = 1_320.0;
pub const PLAY_REFERENCE_HEIGHT: f32 = 796.0;
pub const PLAY_VIEWPORT_WIDTH: f32 = 1_040.0;
pub const PLAY_GAP: f32 = 16.0;
pub const PLAY_SIDE_DECK_WIDTH: f32 = 264.0;

const BG0: Color32 = Color32::from_rgb(0x0E, 0x11, 0x14);
const SURFACE1: Color32 = Color32::from_rgb(0x15, 0x1A, 0x1F);
const SURFACE2: Color32 = Color32::from_rgb(0x1C, 0x23, 0x2A);
const PLASTIC: Color32 = Color32::from_rgb(0xD8, 0xC7, 0xA7);
const TEXT: Color32 = Color32::from_rgb(0xF1, 0xEE, 0xE8);
const MUTED: Color32 = Color32::from_rgb(0x92, 0x98, 0xA0);
const RED: Color32 = Color32::from_rgb(0xD1, 0x4A, 0x36);
const GREEN: Color32 = Color32::from_rgb(0x5C, 0x9B, 0x76);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Boot,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    Ready,
    Waiting,
}

impl CheckState {
    fn label(self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Waiting => "WAITING",
        }
    }

    fn color(self) -> Color32 {
        match self {
            Self::Ready => GREEN,
            Self::Waiting => MUTED,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootStatus {
    pub memory: CheckState,
    pub gpu: CheckState,
    pub sound: CheckState,
    pub game_card: CheckState,
}

impl BootStatus {
    pub const fn first_playable(has_local_game: bool) -> Self {
        Self {
            memory: CheckState::Ready,
            gpu: CheckState::Ready,
            // CPAL keep-alive is T005, so Bundle A must not claim audible output.
            sound: CheckState::Waiting,
            game_card: if has_local_game {
                CheckState::Ready
            } else {
                CheckState::Waiting
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GameViewport {
    pub texture_id: TextureId,
    pub source_size: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeDeckStatus {
    pub game_loaded: bool,
    pub input_live: bool,
}

pub struct FirstPlayableShell {
    boot_started_at: Instant,
    screen: AppScreen,
    boot_status: BootStatus,
}

impl FirstPlayableShell {
    pub fn new(boot_status: BootStatus) -> Self {
        Self::new_at(Instant::now(), boot_status)
    }

    pub fn new_at(boot_started_at: Instant, boot_status: BootStatus) -> Self {
        Self {
            boot_started_at,
            screen: AppScreen::Boot,
            boot_status,
        }
    }

    pub fn screen(&self) -> AppScreen {
        self.screen
    }

    pub fn is_booting(&self) -> bool {
        self.screen == AppScreen::Boot
    }

    pub fn advance(&mut self, now: Instant) {
        if self.screen == AppScreen::Boot
            && now.saturating_duration_since(self.boot_started_at) >= Duration::from_millis(BOOT_DURATION_MS)
        {
            self.screen = AppScreen::Play;
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, viewport: Option<GameViewport>, runtime: RuntimeDeckStatus) {
        self.advance(Instant::now());

        match self.screen {
            AppScreen::Boot => self.show_boot(ui),
            AppScreen::Play => self.show_play(ui, viewport, runtime),
        }
    }

    fn show_boot(&self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(Frame::default().fill(BG0))
            .show(ui, |ui| {
                let available = ui.available_size();
                let top_pad = ((available.y - 360.0) * 0.5).max(24.0);
                ui.add_space(top_pad);

                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("M32").size(48.0).strong().color(PLASTIC));
                    ui.add_space(4.0);
                    ui.label(RichText::new("MOBILE ENTERTAINMENT SYSTEM").size(12.0).color(MUTED));
                    ui.add_space(40.0);

                    boot_check(ui, "MEMORY", self.boot_status.memory);
                    boot_check(ui, "GPU", self.boot_status.gpu);
                    boot_check(ui, "SOUND", self.boot_status.sound);
                    boot_check(ui, "GAME CARD", self.boot_status.game_card);

                    ui.add_space(28.0);
                    ui.label(RichText::new("The console that never existed.").size(12.0).color(MUTED));
                });
            });
    }

    fn show_play(&self, ui: &mut egui::Ui, viewport: Option<GameViewport>, runtime: RuntimeDeckStatus) {
        egui::CentralPanel::default()
            .frame(Frame::default().fill(BG0).inner_margin(24.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("M32").size(20.0).strong().color(PLASTIC));
                    ui.add_space(12.0);
                    ui.label(RichText::new("FIRST PLAYABLE").size(12.0).color(RED));
                });
                ui.add_space(16.0);

                let available = ui.available_size();
                let side_width = PLAY_SIDE_DECK_WIDTH.min((available.x * 0.28).max(220.0));
                let gap = PLAY_GAP;
                let viewport_width = (available.x - gap - side_width).max(240.0);
                let height = available.y.max(320.0);

                ui.horizontal(|ui| {
                    let (viewport_rect, _) = ui.allocate_exact_size(Vec2::new(viewport_width, height), Sense::hover());

                    ui.painter().rect_filled(viewport_rect, 10.0, SURFACE2);
                    ui.painter().rect_stroke(
                        viewport_rect,
                        10.0,
                        Stroke::new(1.0, Color32::from_rgb(0x2A, 0x32, 0x3A)),
                        egui::StrokeKind::Inside,
                    );

                    if let Some(viewport) = viewport {
                        paint_pixel_perfect(ui, viewport_rect, viewport);
                    } else {
                        ui.painter().text(
                            viewport_rect.center(),
                            Align2::CENTER_CENTER,
                            if runtime.game_loaded {
                                "GAME VIEWPORT\nWaiting for first guest frame"
                            } else {
                                "GAME VIEWPORT\nRun with --jad <file.jad> --jar <file.jar>"
                            },
                            FontId::proportional(18.0),
                            MUTED,
                        );
                    }

                    ui.add_space(gap);

                    ui.allocate_ui_with_layout(Vec2::new(side_width, height), Layout::top_down(Align::Min), |ui| {
                        Frame::default()
                            .fill(SURFACE1)
                            .corner_radius(10.0)
                            .inner_margin(16.0)
                            .show(ui, |ui| {
                                ui.set_min_width((side_width - 32.0).max(120.0));

                                ui.label(RichText::new("SYSTEM DECK").size(12.0).strong().color(PLASTIC));
                                ui.add_space(16.0);

                                deck_row(ui, "DISPLAY", if viewport.is_some() { "LIVE" } else { "READY" });
                                deck_row(ui, "INPUT", if runtime.input_live { "LIVE" } else { "WAITING" });
                                deck_row(ui, "AUDIO", "HOST READY");
                                deck_row(ui, "STORAGE", "WIRED");

                                ui.add_space(20.0);
                                ui.separator();
                                ui.add_space(12.0);
                                ui.label(RichText::new("0.1.0 Bundle A").size(13.0).strong().color(TEXT));
                                ui.label(
                                    RichText::new("Local JAD/JAR + live frame + keyboard input")
                                        .size(12.0)
                                        .color(MUTED),
                                );
                            });
                    });
                });
            });
    }
}

pub fn apply_theme(context: &Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG0;
    visuals.window_fill = SURFACE1;
    visuals.extreme_bg_color = SURFACE2;
    visuals.faint_bg_color = SURFACE2;
    visuals.override_text_color = Some(TEXT);
    context.set_visuals(visuals);
}

fn paint_pixel_perfect(ui: &egui::Ui, bounds: Rect, viewport: GameViewport) {
    if viewport.source_size.x <= 0.0 || viewport.source_size.y <= 0.0 {
        return;
    }

    let scale = (bounds.width() / viewport.source_size.x)
        .min(bounds.height() / viewport.source_size.y)
        .floor()
        .max(1.0);
    let size = viewport.source_size * scale;
    let rect = Rect::from_center_size(bounds.center(), size);

    ui.painter().image(
        viewport.texture_id,
        rect,
        Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn boot_check(ui: &mut egui::Ui, name: &str, state: CheckState) {
    ui.horizontal(|ui| {
        ui.set_min_width(260.0);
        ui.label(RichText::new(format!("{name:<12}")).monospace().size(13.0).color(TEXT));
        ui.label(RichText::new(state.label()).monospace().size(13.0).color(state.color()));
    });
}

fn deck_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).size(11.0).color(TEXT));
        });
    });
    ui.add_space(8.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_duration_is_exactly_one_second() {
        assert_eq!(BOOT_DURATION_MS, 1_000);
    }

    #[test]
    fn play_reference_columns_fill_locked_content_width() {
        assert_eq!(
            PLAY_VIEWPORT_WIDTH + PLAY_GAP + PLAY_SIDE_DECK_WIDTH,
            PLAY_REFERENCE_CONTENT_WIDTH
        );
        assert_eq!(PLAY_REFERENCE_HEIGHT, 796.0);
    }

    #[test]
    fn first_playable_boot_status_does_not_claim_cpal_before_t005() {
        let status = BootStatus::first_playable(true);
        assert_eq!(status.memory, CheckState::Ready);
        assert_eq!(status.gpu, CheckState::Ready);
        assert_eq!(status.sound, CheckState::Waiting);
        assert_eq!(status.game_card, CheckState::Ready);
    }

    #[test]
    fn boot_transitions_only_after_locked_duration() {
        let start = Instant::now();
        let mut shell = FirstPlayableShell::new_at(start, BootStatus::first_playable(false));

        shell.advance(start + Duration::from_millis(999));
        assert_eq!(shell.screen(), AppScreen::Boot);

        shell.advance(start + Duration::from_millis(1_000));
        assert_eq!(shell.screen(), AppScreen::Play);
    }

    #[test]
    fn window_geometry_matches_locked_contract() {
        assert_eq!(REFERENCE_WINDOW_WIDTH, 1_440.0);
        assert_eq!(REFERENCE_WINDOW_HEIGHT, 900.0);
        assert_eq!(MIN_WINDOW_WIDTH, 1_180.0);
        assert_eq!(MIN_WINDOW_HEIGHT, 700.0);
    }
}
