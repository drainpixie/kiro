use constants::*;
use eframe::{run_native, NativeOptions};
use egui::Theme;
use nodes::Node;
use render::Kiro;
use std::collections::HashSet;

mod constants;
mod nodes;
mod render;

// TODO: Custom fonts
// TODO: In-app config
// TODO: Better styling

fn main() -> eframe::Result {
    let mut existing_ids = HashSet::new();
    let nodes = vec![
        Node::new(&mut existing_ids, "Timeline", "127.0.0.1:3000"),
        Node::new(&mut existing_ids, "Incubator", "127.0.0.2:3000"),
    ];

    run_native(
        APP_NAME,
        NativeOptions::default(),
        Box::new(|ctx| {
            ctx.egui_ctx.set_theme(Theme::Light);
            Ok(Box::new(Kiro::new(nodes)))
        }),
    )
}
