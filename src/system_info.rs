//! Startup banner — one log entry printed once at `App::new` so the
//! Logger panel lands populated.

use egui_lens::ReactiveEventLogger;

#[cfg(not(target_arch = "wasm32"))]
use sysinfo::System;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn show_system_info(logger: &ReactiveEventLogger) {
    // Lens renders newest-first — log system first, banner second, so
    // banner sits on top visually.
    logger.log_info(&format_system_details());
    logger.log_info(&format_banner());
}

fn format_banner() -> String {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut out = String::new();
    out.push_str(&format!(
        "**** Welcome to grafica-ide — DSL-driven graphics IDE, Version {VERSION}\n"
    ));
    out.push_str(&format!("**** Today is {now}\n"));
    out.push('\n');
    out.push_str("CITIZENS\n");
    out.push_str("    canvas    — egui_grafica\n");
    out.push_str("    editor    — egui_quill\n");
    out.push_str("    logger    — egui_lens\n");
    out.push_str("    project   — (built-in)\n");
    out.push('\n');
    out.push_str("DEPENDENCIES\n");
    out.push_str(&format!("    grafica-ide      : {VERSION}\n"));
    out.push_str("    egui             : 0.34\n");
    out.push_str("    eframe           : 0.34 (wgpu)\n");
    out.push_str("    egui_dock        : 0.19\n");
    out.push_str("    egui_mobius      : 0.4.0 (path)\n");
    out.push_str("    egui_citizen     : 0.4.0 (path)\n");
    out.push_str("    egui_lens        : 0.4.0 (path)\n");
    out.push_str("    egui_quill       : 0.4.0 (path)\n");
    out.push_str("    egui_grafica     : 0.4.0 (path, gpu)");
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn format_system_details() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os = System::long_os_version().unwrap_or_else(|| "unknown".into());
    let kernel = System::kernel_version().unwrap_or_else(|| "unknown".into());
    let host = System::host_name().unwrap_or_else(|| "unknown".into());
    let physical = sys
        .physical_core_count()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "?".into());
    let logical = sys.cpus().len();
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "unknown".into());
    let mem_total_gib = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_used_gib = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

    let mut out = String::new();
    out.push_str("SYSTEM\n");
    out.push_str(&format!("    OS               : {os}\n"));
    out.push_str(&format!("    Kernel           : {kernel}\n"));
    out.push_str(&format!("    Host             : {host}\n"));
    out.push_str(&format!("    CPU              : {cpu_brand}\n"));
    out.push_str(&format!("    Cores            : {physical} physical / {logical} logical\n"));
    out.push_str(&format!(
        "    Memory           : {mem_used_gib:.1} / {mem_total_gib:.1} GiB used"
    ));
    out
}

#[cfg(target_arch = "wasm32")]
fn format_system_details() -> String {
    let mut out = String::from("SYSTEM\n");
    let win = web_sys::window();
    if let Some(w) = win.as_ref() {
        if let Ok(nav) = w.navigator().user_agent() {
            out.push_str(&format!("    UA               : {nav}\n"));
        }
        if let Some(scr) = w.screen().ok() {
            if let (Ok(w), Ok(h)) = (scr.width(), scr.height()) {
                out.push_str(&format!("    Screen           : {w} x {h}\n"));
            }
        }
    }
    out.push_str("    Build            : wasm32-unknown-unknown");
    out
}
