//! Top-level App — Dispatcher, dock layout, ribbon, settings apply.

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher};
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_lens::ReactiveEventLogger;

use crate::panels::{CanvasPanel, ProjectPanel};
use crate::ribbon::RibbonState;
use crate::settings::{Settings, apply_ui_scale};
use crate::state::SharedState;
use crate::system_info;
use crate::tabs::{
    CANVAS_ID, EDITOR_ID, LOGGER_ID, PROJECT_ID, SETTINGS_ID, Tab, TabKind, TabViewer,
};

/// Storage key for the persisted [`Settings`] blob.
const SETTINGS_KEY: &str = "grafica_ide_settings";

pub struct App {
    dispatcher: Dispatcher,
    dock_state: DockState<Tab>,
    state: SharedState,
    ribbon: RibbonState,
    project: ProjectPanel,
    canvas: CanvasPanel,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Restore user settings from disk so UI scale, timezone, clock
        // format, default directory, and grid units survive a restart.
        let saved_settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value::<Settings>(s, SETTINGS_KEY))
            .unwrap_or_default();

        // Citizen registration. Each registered citizen receives a
        // `CitizenState` handle whose clones share storage — panel
        // and dispatcher see the same reactive state.
        let mut dispatcher = Dispatcher::new();
        let project_state = dispatcher.register(CitizenId::new(PROJECT_ID));
        let _editor_state = dispatcher.register(CitizenId::new(EDITOR_ID));
        let canvas_state = dispatcher.register(CitizenId::new(CANVAS_ID));
        let _logger_state = dispatcher.register(CitizenId::new(LOGGER_ID));
        let _settings_state = dispatcher.register(CitizenId::new(SETTINGS_ID));
        dispatcher.activate(&CitizenId::new(EDITOR_ID));
        let _ = dispatcher.drain_messages();

        let state = SharedState::new(saved_settings.clone());

        // Startup banner — newest-first lens means we log details
        // then the welcome banner so the banner sits on top visually.
        {
            let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
            system_info::show_system_info(&logger);
        }

        let project = ProjectPanel::new(project_state);
        // Seed the canvas's starting scene with the user's preferred
        // grid units — mils by default, not pixels.
        let canvas = CanvasPanel::new(canvas_state, saved_settings.grid_units);

        // Dock layout — Settings sits in the Project leaf as a sibling
        // tab so it's visible from launch (Project still active by
        // default; one click on the Settings tab strip switches).
        //
        //   ┌──────────────────┬───────────┬─────────┬───────────┐
        //   │ Project Settings │  Editor   │ Canvas  │ Inspector │
        //   ├──────────────────┴───────────┴─────────┴───────────┤
        //   │                       Logger                       │
        //   └────────────────────────────────────────────────────┘
        let mut dock_state = DockState::new(vec![Tab::new(TabKind::Editor)]);
        let surface = dock_state.main_surface_mut();
        let [editor_node, canvas_node] =
            surface.split_right(NodeIndex::root(), 0.40, vec![Tab::new(TabKind::Canvas)]);
        let [_canvas_node2, _inspector_node] = surface.split_right(
            canvas_node,
            0.75,
            vec![Tab::new(TabKind::Inspector)],
        );
        let [_, _project_node] = surface.split_left(
            editor_node,
            0.22,
            vec![Tab::new(TabKind::Project), Tab::new(TabKind::Settings)],
        );
        let [_, _logger_node] =
            surface.split_below(NodeIndex::root(), 0.72, vec![Tab::new(TabKind::Logger)]);

        Self {
            dispatcher,
            dock_state,
            state,
            ribbon: RibbonState::default(),
            project,
            canvas,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let settings_snapshot = self.state.settings.get();
        apply_ui_scale(ui.ctx(), &settings_snapshot);

        ui.vertical(|ui| {
            crate::ribbon::show(ui, &mut self.ribbon, &settings_snapshot);
            ui.separator();

            // The dock area borrows `dock_state` mutably; the TabViewer
            // borrows other fields of self. To avoid the overlapping
            // self-borrow, swap dock_state + dispatcher out, run the
            // dock, then swap them back.
            let mut dock_state = self.dock_state.clone();
            let mut dispatcher = std::mem::take(&mut self.dispatcher);
            {
                let style = egui_dock::Style::from_egui(ui.ctx().global_style().as_ref());
                let mut viewer = TabViewer {
                    state: &self.state,
                    dispatcher: &mut dispatcher,
                    project: &mut self.project,
                    canvas: &mut self.canvas,
                };
                DockArea::new(&mut dock_state)
                    .style(style)
                    .show_inside(ui, &mut viewer);
            }
            self.dock_state = dock_state;
            self.dispatcher = dispatcher;
        });

        // Modals overlay everything else — drawn last.
        crate::ribbon::show_modals(ui.ctx(), &mut self.ribbon);

        // Tick the ribbon clock and let any debounced DSL eval fire
        // without requiring a fresh input event.
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
    }

    /// Persist user settings through eframe's storage backend. Called
    /// by eframe periodically (default ~30s) and on app close. On
    /// native this lands in a platform-appropriate config path; on
    /// wasm it writes to `localStorage`.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let settings = self.state.settings.get();
        eframe::set_value(storage, SETTINGS_KEY, &settings);
    }
}
