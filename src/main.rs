//! BotW Actor Tool — Rust + egui rewrite of the original Python tool.
//!
//! Format handling is provided by libraries:
//! - `roead` — AAMP, BYML, SARC, Yaz0 (botw / nintendo formats)
//! - `msyt` (+ `msbt`) — MSBT message text archives
//! - game data JSON files are shared with the Python tool

mod actor;
mod actorinfo;
mod app;
mod data;
mod flag;
mod i18n;
mod pack;
mod settings;
mod store;
mod texts;
mod util;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("BotW Actor Tool"),
        ..Default::default()
    };
    eframe::run_native(
        "BotW Actor Tool",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
