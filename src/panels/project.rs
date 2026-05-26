//! Project panel — minimal first cut.
//!
//! Phase 1 surface: an Open File… button that streams the picked
//! file's contents into the editor, plus a small "recently opened"
//! list. The grand library system from zicad lands here later.

use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;
use egui_citizen::{CitizenId, CitizenState};
use egui_lens::ReactiveEventLogger;

use crate::state::SharedState;
use crate::theme::TokyoNight;

pub struct ProjectPanel {
    #[allow(dead_code)]
    citizen_id: CitizenId,
    #[allow(dead_code)]
    citizen_state: CitizenState,
    /// Files the user has opened this session — visible in the panel.
    recent: Vec<String>,
    /// Receives a file picked via the dialog. The dialog runs async
    /// on wasm and on a worker thread on native; `show` drains it
    /// on the next frame.
    file_rx: Option<mpsc::Receiver<(String, String)>>,
}

impl ProjectPanel {
    pub fn new(citizen_state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new(crate::tabs::PROJECT_ID),
            citizen_state,
            recent: Vec::new(),
            file_rx: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        // Drain any newly picked file into the editor.
        let received = self.file_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some((name, content)) = received {
            let mut editor = state.editor.get();
            editor.content = content;
            state.editor.set(editor);
            if !self.recent.iter().any(|n| n == &name) {
                self.recent.insert(0, name.clone());
                self.recent.truncate(10);
            }
            let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
            logger.log_info(&format!("Opened {name}"));
            self.file_rx = None;
        }

        ui.add_space(6.0);
        ui.heading("Project");
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Open a `.canvas` file to edit. Edits parse live into the canvas tab.",
            )
            .small()
            .weak(),
        );
        ui.separator();

        let default_dir = state.settings.get().default_directory.clone();
        if ui.button("📂 Open File…").clicked() {
            let (tx, rx) = mpsc::channel();
            self.file_rx = Some(rx);
            spawn_open_dialog(tx, ui.ctx().clone(), default_dir);
        }
        ui.add_space(8.0);

        ui.collapsing("Recent", |ui| {
            if self.recent.is_empty() {
                ui.label(
                    egui::RichText::new("Nothing opened this session.")
                        .small()
                        .color(TokyoNight::FG_DIM),
                );
            } else {
                for name in &self.recent {
                    ui.label(egui::RichText::new(name).color(TokyoNight::FG));
                }
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_open_dialog(
    tx: mpsc::Sender<(String, String)>,
    ctx: egui::Context,
    default_dir: Option<PathBuf>,
) {
    std::thread::spawn(move || {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("graphica node", &["gnx"])
            .add_filter("All files", &["*"]);
        if let Some(dir) = default_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(path) = dialog.pick_file()
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            let _ = tx.send((name, content));
            ctx.request_repaint();
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn spawn_open_dialog(
    tx: mpsc::Sender<(String, String)>,
    ctx: egui::Context,
    _default_dir: Option<PathBuf>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("graphica node", &["gnx"])
            .pick_file()
            .await
        {
            let name = handle.file_name();
            let bytes = handle.read().await;
            if let Ok(content) = String::from_utf8(bytes) {
                let _ = tx.send((name, content));
                ctx.request_repaint();
            }
        }
    });
}
