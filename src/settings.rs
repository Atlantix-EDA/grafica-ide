//! Application settings — UI scale, timezone, clock format.
//!
//! Settings live in `SharedState::settings` as a `Dynamic<Settings>`;
//! the settings panel mutates them reactively and other surfaces
//! (ribbon clock, ui-scale apply) observe through `.get()`.

use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// UI scale factor multiplied into the platform's native DPI.
    pub ui_scale: f32,
    /// IANA timezone name (e.g. "US/Eastern"). `None` → `chrono::Local`.
    pub user_timezone: Option<String>,
    /// `true` → ribbon clock renders 24-hour; `false` → 12-hour AM/PM.
    pub use_24_hour_clock: bool,
    /// Default directory for Open File / Save dialogs. `None` →
    /// platform default (last-used or home).
    pub default_directory: Option<std::path::PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            user_timezone: None,
            use_24_hour_clock: false,
            default_directory: None,
        }
    }
}

pub const UI_SCALE_MIN: f32 = 0.7;
pub const UI_SCALE_MAX: f32 = 2.0;
pub const UI_SCALE_STEP: f32 = 0.05;

impl Settings {
    pub fn pixels_per_point(&self, native: f32) -> f32 {
        (native * self.ui_scale).clamp(0.5, 4.0)
    }
}

pub fn apply_ui_scale(ctx: &egui::Context, settings: &Settings) {
    let native = ctx.native_pixels_per_point().unwrap_or(1.0);
    let target = settings.pixels_per_point(native);
    if (ctx.pixels_per_point() - target).abs() > 1e-3 {
        ctx.set_pixels_per_point(target);
    }
}
