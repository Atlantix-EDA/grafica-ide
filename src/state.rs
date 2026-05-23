//! Shared, cross-panel reactive state.
//!
//! Each `Dynamic<T>` is a reactive cell observable by every citizen
//! that holds a clone of the handle. The editor's text, the log
//! buffer, log colour scheme, and the application settings all live
//! here. The canvas owns its own `Registry` (a `Dynamic<Scene>`);
//! the canvas panel reads the editor's content and pushes parsed
//! Scenes through that Registry on the DSL side.

use egui_lens::{LogColors, ReactiveEventLoggerState};
use egui_mobius_reactive::Dynamic;
use egui_quill::ReactiveEditorState;

use crate::settings::Settings;

/// Starter DSL displayed when the app first opens. Comment-only so a
/// fresh canvas is empty and the user types into a real prompt rather
/// than a populated example. Once we have a `.canvas` syntect syntax
/// the keywords will highlight.
pub const STARTER_DSL: &str = r#"// Welcome to grafica-ide.
//
// This pane edits the canvas DSL — Visio/draw.io-style diagrams and
// node graphs, expressed as text. Edits parse on a short debounce
// and update the canvas tab. Parse errors land in the Logger tab.
//
// A minimal scene to get started:
//
//   canvas Untitled {
//       settings { grid_spacing 20.0 snap_to_grid on show_grid on
//                  default_routing orthogonal }
//   }
//
// Or use the shape ribbon on the canvas tab to draw and let the
// editor reflect the result (round-trip coming next).
"#;

/// Reactive cells shared across panels. Cloning a `Dynamic<T>` shares
/// the underlying storage.
pub struct SharedState {
    pub editor: Dynamic<ReactiveEditorState>,
    pub log: Dynamic<ReactiveEventLoggerState>,
    pub log_colors: Dynamic<LogColors>,
    pub settings: Dynamic<Settings>,
}

impl SharedState {
    /// Build the shared state. `settings` is the application
    /// preferences restored from `eframe::Storage` on app startup —
    /// `Settings::default()` if there's no saved state yet.
    pub fn new(settings: Settings) -> Self {
        Self {
            editor: Dynamic::new(
                ReactiveEditorState::new()
                    .with_content(STARTER_DSL)
                    // No bundled `.canvas` syntect grammar yet, so we
                    // skip the language picker. base16-ocean.dark
                    // pairs with the Tokyo Night chrome.
                    .with_theme("base16-ocean.dark"),
            ),
            log: Dynamic::new(ReactiveEventLoggerState::new()),
            log_colors: {
                let mut colors = LogColors::default();
                colors.set_custom_color("citizen", egui::Color32::from_rgb(100, 200, 255));
                colors.set_custom_color("backend", egui::Color32::from_rgb(140, 220, 140));
                Dynamic::new(colors)
            },
            settings: Dynamic::new(settings),
        }
    }
}
