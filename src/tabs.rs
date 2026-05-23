//! Tab definitions and the `egui_dock::TabViewer` bridge.
//!
//! Five tabs: Project, Editor, Canvas, Logger, Settings. The first
//! four are part of the default layout; Settings opens on demand from
//! the ribbon. Tab clicks call `dispatcher.activate(...)` so the
//! citizen one-hot lifecycle stays accurate end-to-end.

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher};
use egui_lens::ReactiveEventLogger;
use egui_quill::ReactiveEditor;

use crate::panels::{CanvasPanel, ProjectPanel};
use crate::state::SharedState;

pub const PROJECT_ID: &str = "project";
pub const EDITOR_ID: &str = "editor";
pub const CANVAS_ID: &str = "canvas";
pub const LOGGER_ID: &str = "logger";
pub const SETTINGS_ID: &str = "settings";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    Project,
    Editor,
    Canvas,
    Logger,
    Settings,
}

#[derive(Clone)]
pub struct Tab {
    pub kind: TabKind,
}

impl Tab {
    pub fn new(kind: TabKind) -> Self {
        Self { kind }
    }

    pub fn title(&self) -> &'static str {
        match self.kind {
            TabKind::Project => "Project",
            TabKind::Editor => "Editor",
            TabKind::Canvas => "Canvas",
            TabKind::Logger => "Logger",
            TabKind::Settings => "Settings",
        }
    }

    pub fn citizen_id(&self) -> CitizenId {
        CitizenId::new(match self.kind {
            TabKind::Project => PROJECT_ID,
            TabKind::Editor => EDITOR_ID,
            TabKind::Canvas => CANVAS_ID,
            TabKind::Logger => LOGGER_ID,
            TabKind::Settings => SETTINGS_ID,
        })
    }
}

pub struct TabViewer<'a> {
    pub state: &'a SharedState,
    pub dispatcher: &'a mut Dispatcher,
    pub project: &'a mut ProjectPanel,
    pub canvas: &'a mut CanvasPanel,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if response.clicked() {
            self.dispatcher.activate(&tab.citizen_id());
        }
    }

    /// Settings is always reachable from the default layout, so the
    /// close-X is hidden — a stray click can't lose it. Every other
    /// tab keeps the default close behaviour.
    fn closeable(&mut self, tab: &mut Tab) -> bool {
        !matches!(tab.kind, TabKind::Settings)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab.kind {
            TabKind::Project => self.project.show(ui, self.state),
            TabKind::Editor => {
                // The Quill editor body fills the tab; language /
                // theme pickers live in Settings, not in the panel.
                let editor = ReactiveEditor::new(&self.state.editor).with_pickers(false);
                editor.show(ui);
            }
            TabKind::Canvas => self.canvas.show(ui, self.state),
            TabKind::Logger => {
                // Force the panel to claim the full available rect —
                // lens's internal scroll would otherwise auto-shrink.
                let avail = ui.available_size_before_wrap();
                ui.allocate_ui_with_layout(
                    avail,
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let logger = ReactiveEventLogger::with_colors(
                            &self.state.log,
                            &self.state.log_colors,
                        );
                        logger.show(ui);
                    },
                );
            }
            TabKind::Settings => crate::panels::settings::show(ui, self.state),
        }
    }
}
