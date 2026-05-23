//! Top-level App — Dispatcher, dock layout, ribbon, settings apply.

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher};
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_lens::ReactiveEventLogger;

use crate::panels::{CanvasPanel, ProjectPanel};
use crate::ribbon::RibbonState;
use crate::settings::apply_ui_scale;
use crate::state::SharedState;
use crate::system_info;
use crate::tabs::{
    CANVAS_ID, EDITOR_ID, LOGGER_ID, PROJECT_ID, SETTINGS_ID, Tab, TabKind, TabViewer,
};

pub struct App {
    dispatcher: Dispatcher,
    dock_state: DockState<Tab>,
    state: SharedState,
    ribbon: RibbonState,
    project: ProjectPanel,
    canvas: CanvasPanel,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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

        let state = SharedState::new();

        // Startup banner — newest-first lens means we log details
        // then the welcome banner so the banner sits on top visually.
        {
            let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
            system_info::show_system_info(&logger);
        }

        let project = ProjectPanel::new(project_state);
        let canvas = CanvasPanel::new(canvas_state);

        // Dock layout:
        //
        //   ┌──────────┬───────────┬───────────────┐
        //   │ Project  │  Editor   │   Canvas      │
        //   ├──────────┴───────────┴───────────────┤
        //   │                Logger                │
        //   └──────────────────────────────────────┘
        let mut dock_state = DockState::new(vec![Tab::new(TabKind::Editor)]);
        let surface = dock_state.main_surface_mut();
        let [editor_node, _canvas_node] =
            surface.split_right(NodeIndex::root(), 0.45, vec![Tab::new(TabKind::Canvas)]);
        let [_, _project_node] =
            surface.split_left(editor_node, 0.22, vec![Tab::new(TabKind::Project)]);
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

/// Focus the existing Settings tab if one exists; otherwise push a
/// fresh Settings tab onto the focused leaf.
fn open_or_focus_settings(dock_state: &mut DockState<Tab>) {
    if let Some(path) = dock_state.find_tab_from(|t| matches!(t.kind, TabKind::Settings)) {
        let _ = dock_state.set_active_tab(path);
        return;
    }
    dock_state.push_to_focused_leaf(Tab::new(TabKind::Settings));
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let settings_snapshot = self.state.settings.get();
        apply_ui_scale(ui.ctx(), &settings_snapshot);

        ui.vertical(|ui| {
            crate::ribbon::show(ui, &mut self.ribbon, &settings_snapshot);
            ui.separator();

            // The ribbon's Settings click takes effect this frame, so
            // the dock reflects it without a one-frame delay.
            if std::mem::take(&mut self.ribbon.open_settings_requested) {
                open_or_focus_settings(&mut self.dock_state);
                self.dispatcher.activate(&CitizenId::new(SETTINGS_ID));
                let _ = self.dispatcher.drain_messages();
            }

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
}
