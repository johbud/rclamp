#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
use pretty_env_logger;

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    pretty_env_logger::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Rclamp",
        native_options,
        Box::new(|cc| Box::new(rclamp::Rclamp::new(cc))),
    )
}
