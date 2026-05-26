//! Settings panel — UI scale, timezone, clock format, default
//! directory.

use eframe::egui;
use egui_grafica::{CanvasBackground, GridUnits};
use egui_grafica::model::PortMarkerStyle;
use egui_lens::ReactiveEventLogger;

use crate::settings::{Settings, UI_SCALE_MAX, UI_SCALE_MIN, UI_SCALE_STEP};
use crate::state::SharedState;
use crate::theme::TokyoNight;

const COMMON_TIMEZONES: &[&str] = &[
    "UTC",
    "US/Eastern",
    "US/Central",
    "US/Mountain",
    "US/Pacific",
    "Europe/London",
    "Europe/Paris",
    "Europe/Berlin",
    "Asia/Tokyo",
    "Asia/Shanghai",
    "Australia/Sydney",
];

pub fn show(ui: &mut egui::Ui, state: &SharedState) {
    let mut settings = state.settings.get();
    let before = settings.clone();

    ui.add_space(6.0);
    ui.heading("Application Settings");
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("UI scale, timezone, and file defaults. Changes apply immediately.")
            .small()
            .weak(),
    );
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        display_section(ui, &mut settings);
        ui.add_space(16.0);
        units_section(ui, &mut settings);
        ui.add_space(16.0);
        time_section(ui, &mut settings);
        ui.add_space(16.0);
        files_section(ui, &mut settings);
    });

    if settings != before {
        let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
        log_diff(&logger, &before, &settings);
        state.settings.set(settings);
    }
}

fn display_section(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Display").strong().color(TokyoNight::BLUE));
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("UI scale:");
            ui.add(
                egui::Slider::new(&mut settings.ui_scale, UI_SCALE_MIN..=UI_SCALE_MAX)
                    .step_by(UI_SCALE_STEP as f64)
                    .fixed_decimals(2)
                    .text("×"),
            );
            if ui.button("Reset").clicked() {
                settings.ui_scale = 1.0;
            }
        });
        ui.label(
            egui::RichText::new(
                "Multiplies the platform's native DPI scale. 1.00× is native.",
            )
            .small()
            .weak(),
        );
    });
}

fn units_section(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.group(|ui| {
        ui.label(
            egui::RichText::new("Units")
                .strong()
                .color(TokyoNight::BLUE),
        );
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Default grid units:");
            egui::ComboBox::from_id_salt("settings_grid_units_combo")
                .selected_text(grid_units_label(settings.grid_units))
                .show_ui(ui, |ui| {
                    for u in [GridUnits::Pixels, GridUnits::Mils, GridUnits::Millimeters, GridUnits::Inches] {
                        ui.selectable_value(&mut settings.grid_units, u, grid_units_label(u));
                    }
                });
        });
        ui.label(
            egui::RichText::new(
                "Applied to a new canvas at startup. Loaded `.canvas` files keep their own units; \
                 the canvas ribbon's units picker overrides per scene.",
            )
            .small()
            .weak(),
        );

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new("Appearance")
                .strong()
                .color(TokyoNight::BLUE),
        );
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Default canvas background:");
            egui::ComboBox::from_id_salt("settings_default_bg_combo")
                .selected_text(settings.default_background.label())
                .show_ui(ui, |ui| {
                    use CanvasBackground::*;
                    for bg in [Light, Slate, Charcoal, Dark] {
                        ui.selectable_value(&mut settings.default_background, bg, bg.label());
                    }
                });
        });
        ui.label(
            egui::RichText::new(
                "Tone of a fresh canvas at startup. Per-scene override lives on the canvas ribbon.",
            )
            .small()
            .weak(),
        );

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new("Ports / Connections")
                .strong()
                .color(TokyoNight::BLUE),
        );
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Marker size:");
            ui.add(
                egui::Slider::new(&mut settings.port_marker_size, 1.0..=12.0)
                    .fixed_decimals(1)
                    .suffix(" px"),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Marker style:");
            egui::ComboBox::from_id_salt("settings_port_marker_style")
                .selected_text(settings.port_marker_style.label())
                .show_ui(ui, |ui| {
                    for s in [
                        PortMarkerStyle::Disc,
                        PortMarkerStyle::Ring,
                        PortMarkerStyle::Cross,
                    ] {
                        ui.selectable_value(&mut settings.port_marker_style, s, s.label());
                    }
                });
        });
        ui.label(
            egui::RichText::new(
                "Applies to every visible port immediately — changes flow into the active canvas this frame.",
            )
            .small()
            .weak(),
        );
    });
}

fn grid_units_label(u: GridUnits) -> &'static str {
    match u {
        GridUnits::Pixels => "Pixels",
        GridUnits::Mils => "Mils",
        GridUnits::Millimeters => "Millimeters",
        GridUnits::Inches => "Inches",
    }
}

fn time_section(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.group(|ui| {
        ui.label(
            egui::RichText::new("Time & Localization")
                .strong()
                .color(TokyoNight::BLUE),
        );
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Timezone:");
            let current = settings.user_timezone.as_deref().unwrap_or("System local");
            egui::ComboBox::from_id_salt("settings_timezone_combo")
                .selected_text(current)
                .width(260.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.user_timezone, None, "System local");
                    ui.separator();
                    ui.label(egui::RichText::new("Common").weak().small());
                    for tz_name in COMMON_TIMEZONES {
                        ui.selectable_value(
                            &mut settings.user_timezone,
                            Some((*tz_name).to_string()),
                            *tz_name,
                        );
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("All").weak().small());
                    for tz in chrono_tz::TZ_VARIANTS {
                        let name = tz.name();
                        ui.selectable_value(
                            &mut settings.user_timezone,
                            Some(name.to_string()),
                            name,
                        );
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Clock format:");
            ui.selectable_value(&mut settings.use_24_hour_clock, true, "24-hour");
            ui.selectable_value(&mut settings.use_24_hour_clock, false, "12-hour");
        });
    });
}

fn files_section(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.group(|ui| {
        ui.label(
            egui::RichText::new("Files")
                .strong()
                .color(TokyoNight::BLUE),
        );
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Default directory:");
            let label = settings
                .default_directory
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(platform default)".to_string());
            ui.label(
                egui::RichText::new(label)
                    .monospace()
                    .color(TokyoNight::FG_DIM),
            );
        });

        ui.horizontal(|ui| {
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Choose…").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_folder()
            {
                settings.default_directory = Some(path);
            }
            if settings.default_directory.is_some() && ui.button("Clear").clicked() {
                settings.default_directory = None;
            }
        });
        ui.label(
            egui::RichText::new("Used as the starting directory for Open File / Save dialogs.")
                .small()
                .weak(),
        );
    });
}

fn log_diff(logger: &ReactiveEventLogger, before: &Settings, after: &Settings) {
    if (before.ui_scale - after.ui_scale).abs() > 1e-3 {
        logger.log_info(&format!(
            "UI scale: {:.2}× → {:.2}×",
            before.ui_scale, after.ui_scale
        ));
    }
    if before.user_timezone != after.user_timezone {
        let to = after.user_timezone.as_deref().unwrap_or("System local");
        logger.log_info(&format!("Timezone → {to}"));
    }
    if before.use_24_hour_clock != after.use_24_hour_clock {
        let to = if after.use_24_hour_clock { "24-hour" } else { "12-hour" };
        logger.log_info(&format!("Clock format → {to}"));
    }
    if before.default_directory != after.default_directory {
        let to = after
            .default_directory
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(platform default)".into());
        logger.log_info(&format!("Default directory → {to}"));
    }
    if before.grid_units != after.grid_units {
        logger.log_info(&format!(
            "Default grid units → {} (applies to new canvases on next startup)",
            grid_units_label(after.grid_units)
        ));
    }
    if before.default_background != after.default_background {
        logger.log_info(&format!(
            "Default canvas background → {} (applies to new canvases on next startup)",
            after.default_background.label()
        ));
    }
    if (before.port_marker_size - after.port_marker_size).abs() > 1e-3 {
        logger.log_info(&format!(
            "Port marker size → {:.1} px",
            after.port_marker_size
        ));
    }
    if before.port_marker_style != after.port_marker_style {
        logger.log_info(&format!(
            "Port marker style → {}",
            after.port_marker_style.label()
        ));
    }
}
