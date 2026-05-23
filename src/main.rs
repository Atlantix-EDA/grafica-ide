//! grafica-ide — DSL-driven graphics IDE.
//!
//! Application entry. Installs egui_grafica's icon font, applies the
//! Tokyo Night Storm theme, registers the canvas wgpu pipeline, and
//! hands off to [`app::App`].

mod app;
mod panels;
mod ribbon;
mod settings;
mod state;
mod system_info;
mod tabs;
mod theme;

use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "grafica-ide",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            // egui_grafica's phosphor icon font powers the canvas
            // ribbon glyphs (shape tools, dock menu).
            egui_grafica::install_fonts(&cc.egui_ctx);
            theme::apply_visuals(&cc.egui_ctx);
            // Register the GPU canvas pipeline. Only present on the
            // wgpu backend — eframe's default renderer here.
            if let Some(rs) = cc.wgpu_render_state.as_ref() {
                egui_grafica::gpu::init(rs);
            }
            Ok(Box::new(app::App::new(cc)))
        }),
    )
}

// ── wasm entrypoint ────────────────────────────────────────────────
//
// Web build is best-effort — eframe's wgpu backend on web is workable
// but rougher than native, and `rfd` degrades to a browser picker.
// Provided so the build doesn't drift; full QA is desktop-first.

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Info).ok();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("no document");
        let canvas = document
            .get_element_by_id("grafica_ide_canvas")
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("missing <canvas id='grafica_ide_canvas'>");

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    egui_grafica::install_fonts(&cc.egui_ctx);
                    theme::apply_visuals(&cc.egui_ctx);
                    if let Some(rs) = cc.wgpu_render_state.as_ref() {
                        egui_grafica::gpu::init(rs);
                    }
                    Ok(Box::new(app::App::new(cc)))
                }),
            )
            .await
            .expect("eframe::WebRunner failed to start");
    });
}
