use eframe::{run_native, App, NativeOptions};
use egui::{CentralPanel, ScrollArea, Theme};

const APP_NAME: &str = "Kiro";

struct Kiro<'a> {
    nodes: Vec<&'a str>,
}

// TODO: Custom fonts
// TODO: In-app config
// TODO: Better styling

impl<'a> Kiro<'a> {
    fn new(nodes: Vec<&'a str>) -> Self {
        Self { nodes }
    }

    fn render_nodes(&self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            self.nodes.iter().for_each(|node| {
                ui.label(*node);
            })
        });
    }
}

impl App for Kiro<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.render_nodes(ui);
        });
    }
}

fn main() -> eframe::Result {
    let nodes = vec![
        "127.0.0.1:3000",
        "127.0.0.1:3001",
        "127.0.0.1:3002",
        "127.0.0.1:3003",
        "127.0.0.1:3004",
        "127.0.0.1:3005",
        "127.0.0.1:3006",
        "127.0.0.1:3007",
        "127.0.0.1:3008",
        "127.0.0.1:3009",
        "127.0.0.1:3010",
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
