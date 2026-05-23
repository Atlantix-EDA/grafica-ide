//! Top ribbon — same layout shape as zicad / CopperForge.
//!
//! Left: app wordmark + File / Info / Settings buttons. Right
//! (right-to-left layout): Hotkeys menu, live clock, clickable
//! version chips that open About-style modals.

use eframe::egui;

use crate::settings::Settings;
use crate::theme::TokyoNight;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct RibbonState {
    pub show_about_modal: bool,
    pub show_grafica_modal: bool,
}

pub fn show(ui: &mut egui::Ui, ribbon: &mut RibbonState, settings: &Settings) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.add_space(4.0);

        ui.label(
            egui::RichText::new("grafica-ide")
                .strong()
                .color(TokyoNight::BLUE),
        );
        ui.separator();

        // File / Info are placeholders; real menu work follows.
        // Settings used to live here as a button — replaced by the
        // permanent Settings tab in the Project leaf.
        ui.label(egui::RichText::new("File").color(TokyoNight::FG_DIM));
        ui.label(egui::RichText::new("Info").color(TokyoNight::FG_DIM));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(4.0);
            show_chips_and_clock(ui, ribbon, settings);
            ui.separator();
            show_hotkeys_menu(ui);
        });
    });
}

fn show_chips_and_clock(ui: &mut egui::Ui, ribbon: &mut RibbonState, settings: &Settings) {
    if ui
        .button(egui::RichText::new(format!("grafica-ide v{VERSION}")).color(TokyoNight::BLUE))
        .on_hover_text("About grafica-ide")
        .clicked()
    {
        ribbon.show_about_modal = true;
    }

    ui.separator();

    if ui
        .button(egui::RichText::new("egui_grafica").color(TokyoNight::GREEN))
        .on_hover_text("Canvas citizen — egui_grafica from egui_mobius")
        .clicked()
    {
        ribbon.show_grafica_modal = true;
    }

    ui.separator();

    let clock_text = render_clock(settings);
    ui.label(egui::RichText::new(clock_text).color(TokyoNight::FG));
}

fn render_clock(settings: &Settings) -> String {
    let time_fmt = if settings.use_24_hour_clock { "%H:%M:%S" } else { "%I:%M:%S %p" };
    match settings.user_timezone.as_deref().and_then(|n| n.parse::<chrono_tz::Tz>().ok()) {
        Some(tz) => {
            let now = chrono::Local::now().with_timezone(&tz);
            format!("{} 🕐 {}", now.format("%Y-%m-%d"), now.format(time_fmt))
        }
        None => {
            let now = chrono::Local::now();
            format!("{} 🕐 {}", now.format("%Y-%m-%d"), now.format(time_fmt))
        }
    }
}

fn show_hotkeys_menu(ui: &mut egui::Ui) {
    ui.menu_button(
        egui::RichText::new("📋 Hotkeys").color(TokyoNight::FG_DIM),
        |ui| {
            ui.heading("Keyboard Shortcuts");
            ui.separator();
            // Hotkeys are inherited from egui_grafica's canvas — see
            // its `Keys` menu inside the canvas tab for the full set.
            for (key, desc) in [
                ("G", "Toggle grid (cursor over canvas)"),
                ("X", "Mirror about X axis"),
                ("Y", "Mirror about Y axis"),
                ("R", "Rotate 90° clockwise"),
                ("Del", "Delete selection"),
                ("Esc", "Disarm active shape tool"),
                ("Middle-drag", "Pan the canvas viewport"),
                ("Wheel", "Zoom (cursor over canvas)"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(key)
                            .monospace()
                            .strong()
                            .color(TokyoNight::CYAN),
                    );
                    ui.label("—");
                    ui.label(egui::RichText::new(desc).color(TokyoNight::FG));
                });
            }
        },
    );
}

pub fn show_modals(ctx: &egui::Context, ribbon: &mut RibbonState) {
    if ribbon.show_about_modal {
        show_about_modal(ctx, ribbon);
    }
    if ribbon.show_grafica_modal {
        egui::Window::new("egui_grafica")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("egui_grafica").color(TokyoNight::GREEN));
                ui.label("Programmable graphics canvas citizen for egui_mobius —");
                ui.label("system block diagrams and node graphs from one model,");
                ui.label("authored in a declarative `.canvas` DSL.");
                ui.add_space(8.0);
                ui.hyperlink("https://github.com/saturn77/egui_mobius/tree/master/crates/egui_grafica");
                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    ribbon.show_grafica_modal = false;
                }
            });
    }
}

fn show_about_modal(ctx: &egui::Context, ribbon: &mut RibbonState) {
    egui::Window::new("About grafica-ide")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([520.0, 360.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("grafica-ide")
                        .color(TokyoNight::BLUE)
                        .size(28.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("v{VERSION}"))
                        .color(TokyoNight::FG_DIM)
                        .monospace(),
                );
                ui.add_space(8.0);
                ui.label("DSL-driven graphics IDE — Visio / draw.io-style diagrams,");
                ui.label("scriptable canvas, and node-graph editing.");
                ui.add_space(8.0);
                ui.label(egui::RichText::new("James Bonanno — Atlantix EDA").color(TokyoNight::FG));
                ui.hyperlink_to(
                    egui::RichText::new("github.com/Atlantix-EDA/grafica-ide").color(TokyoNight::CYAN),
                    "https://github.com/Atlantix-EDA/grafica-ide",
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Built with egui_mobius citizens + Tokyo Night Storm")
                        .color(TokyoNight::FG_DIM)
                        .small(),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let spacer = (ui.available_width() - 360.0).max(0.0) * 0.5;
                    ui.add_space(spacer);
                    for (label, url) in [
                        ("egui_mobius", "https://github.com/saturn77/egui_mobius"),
                        ("egui_grafica", "https://github.com/saturn77/egui_mobius/tree/master/crates/egui_grafica"),
                        ("egui", "https://github.com/emilk/egui"),
                    ] {
                        ui.hyperlink_to(
                            egui::RichText::new(label).color(TokyoNight::CYAN).small(),
                            url,
                        );
                        ui.label(egui::RichText::new("·").color(TokyoNight::FG_DIM).small());
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    ribbon.show_about_modal = false;
                }
            });
        });
}
