use constants::*;
use eframe::{run_native, NativeOptions};
use egui::Theme;
use kiro::{Kiro, Node};
use std::collections::HashSet;

mod constants;
mod kiro;
mod websocket;

// TODO: Custom fonts
// TODO: In-app config
// TODO: Better styling

#[tokio::main(flavor = "multi_thread")]
async fn main() -> eframe::Result {
    twink::log::setup_level(log::LevelFilter::Debug);

    let mut existing_ids = HashSet::new();
    let nodes = vec![
        Node::new(&mut existing_ids, "Timeline", "ws://127.0.0.1:3000"),
        Node::new(&mut existing_ids, "Incubator", "ws://127.0.0.2:3000"),
    ];

    run_native(
        APP_NAME,
        NativeOptions::default(),
        Box::new(|ctx| {
            ctx.egui_ctx.set_theme(Theme::Light);
            Ok(Box::new(Kiro::new(&nodes)))
        }),
    )
}
