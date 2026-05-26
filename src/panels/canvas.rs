//! Canvas panel — wraps [`egui_grafica::CanvasCitizen`] and runs the
//! editor → canvas DSL pipeline.
//!
//! Each frame: hash the editor's current content and compare to what
//! we last *saw* and last *evaluated*. A change schedules the next
//! eval at `now + EVAL_DEBOUNCE`; when that deadline elapses (and
//! the content still differs from the last evaluated content) we
//! parse the DSL and either push the new `Scene` into the canvas
//! `Registry` or log the parse error.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use eframe::egui;
use egui_citizen::{CitizenId, CitizenState};
use egui_grafica::{CanvasBackground, CanvasCitizen, GridUnits, Scene};
use egui_lens::ReactiveEventLogger;
use web_time::{Duration, Instant};

use crate::state::SharedState;

/// Wait this long after the last keystroke before re-parsing the DSL.
/// Short enough that edits feel responsive, long enough that typing
/// a multi-character identifier doesn't fire a parse every keystroke.
const EVAL_DEBOUNCE: Duration = Duration::from_millis(300);

pub struct CanvasPanel {
    #[allow(dead_code)] // Wired through to the Dispatcher; future hooks read it.
    citizen_id: CitizenId,
    #[allow(dead_code)] // Same — kept on the panel so on_activate/click can run.
    citizen_state: CitizenState,

    /// The canvas widget itself. Owns its own `Registry` (the
    /// reactive `Scene` cell). We push parsed Scenes into it.
    canvas: CanvasCitizen,

    /// Hash of the editor content the previous frame — used to detect
    /// "the user typed."
    last_seen_hash: u64,
    /// Hash of the editor content we last fed through the parser —
    /// used to suppress redundant evals after debounce expires on
    /// already-evaluated input.
    last_evaled_hash: u64,
    /// When debounce will fire and the next parse should run.
    next_eval_at: Option<Instant>,
}

impl CanvasPanel {
    /// Build the canvas panel. `initial_units` + `initial_background`
    /// seed the starting empty Scene's units and canvas tone — both
    /// wired from the user's persisted Settings so a fresh document
    /// opens with their preferences. Loaded `.canvas` files override
    /// these with their own values; the ribbon's per-scene pickers
    /// still let the user change them mid-session.
    pub fn new(
        citizen_state: CitizenState,
        initial_units: GridUnits,
        initial_background: CanvasBackground,
    ) -> Self {
        let mut scene = Scene::default();
        scene.settings.grid_units = initial_units;
        scene.settings.background = initial_background;
        let canvas = CanvasCitizen::new(scene);
        Self {
            citizen_id: CitizenId::new(crate::tabs::CANVAS_ID),
            citizen_state,
            canvas,
            last_seen_hash: 0,
            last_evaled_hash: 0,
            next_eval_at: None,
        }
    }

    /// Borrow the underlying canvas citizen — used by the Inspector
    /// tab so it can read the live selection + registry without
    /// reaching through dock state.
    pub fn citizen(&self) -> &CanvasCitizen {
        &self.canvas
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        // Sync port-marker preferences from the app Settings into the
        // current scene. Only push when something actually changed so
        // the registry doesn't snapshot a new undo step every frame.
        {
            let s = state.settings.get();
            let scene_settings = self.canvas.registry.with_scene(|sc| sc.settings.clone());
            let needs_size =
                (scene_settings.port_marker_size - s.port_marker_size).abs() > 1e-3;
            let needs_style = scene_settings.port_marker_style != s.port_marker_style;
            if needs_size || needs_style {
                let mut next = scene_settings;
                next.port_marker_size = s.port_marker_size;
                next.port_marker_style = s.port_marker_style;
                self.canvas.registry.update_settings(next);
            }
        }

        // ── DSL → Scene pipeline ────────────────────────────────────
        let content = state.editor.get().content.clone();
        let hash = hash_str(&content);
        let now = Instant::now();

        // Detect a fresh edit since the previous frame.
        if hash != self.last_seen_hash {
            self.last_seen_hash = hash;
            self.next_eval_at = Some(now + EVAL_DEBOUNCE);
        }

        // Fire the parse once the debounce settles AND the content
        // still differs from what we last evaluated.
        if let Some(deadline) = self.next_eval_at
            && now >= deadline
            && hash != self.last_evaled_hash
        {
            self.next_eval_at = None;
            self.last_evaled_hash = hash;
            let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
            match egui_grafica::lang::parse(&content) {
                Ok(scene) => {
                    self.canvas.registry.set_scene(scene);
                    logger.log_info("Canvas: DSL parsed; scene updated.");
                }
                Err(err) => {
                    logger.log_warning(&format!("Canvas DSL parse error: {err}"));
                }
            }
        }

        // ── Render the canvas ───────────────────────────────────────
        self.canvas.show(ui);

        // ── Scene → DSL sync on save ───────────────────────────────
        // After a Ctrl+S (or File → Save) the canvas hands us the
        // freshly pretty-printed text. Push it into the editor so
        // the source view matches what was just written to disk,
        // and remember its hash as already-evaluated so the eval
        // loop doesn't immediately re-parse and clobber the scene.
        if let Some(dsl) = self.canvas.take_pending_dsl() {
            let new_hash = hash_str(&dsl);
            let mut editor = state.editor.get();
            editor.content = dsl;
            state.editor.set(editor);
            self.last_seen_hash = new_hash;
            self.last_evaled_hash = new_hash;
            self.next_eval_at = None;
            let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
            logger.log_info("Canvas → DSL: editor refreshed from saved scene.");
        }
    }
}

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
