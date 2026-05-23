//! Application settings — UI scale, timezone, clock format, units.
//!
//! Settings live in `SharedState::settings` as a `Dynamic<Settings>`;
//! the settings panel mutates them reactively, other surfaces (ribbon
//! clock, UI-scale apply) observe through `.get()`, and the whole
//! struct is serialised through `eframe::Storage` so user preferences
//! survive a restart.

use eframe::egui;
use egui_grafica::GridUnits;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
    /// Preferred grid units for a *new* canvas. Loaded `.canvas` files
    /// keep their own units; this only seeds the empty starting scene.
    /// The canvas ribbon's per-scene units picker still overrides.
    pub grid_units: GridUnits,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            user_timezone: None,
            use_24_hour_clock: false,
            default_directory: None,
            // Mils default — engineering tooling shouldn't open in pixels.
            grid_units: GridUnits::Mils,
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
