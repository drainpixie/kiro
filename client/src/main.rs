use std::collections::HashSet;

use eframe::{run_native, App, NativeOptions};
use egui::{
    text::LayoutJob, Align, Button, CentralPanel, FontSelection, RichText, ScrollArea, Separator,
    Theme,
};
use rand::Rng;

const APP_NAME: &str = "Kiro";
const PADDING: f32 = 8.0;

struct Node<'a> {
    id: i32,

    hostname: &'a str,
    address: &'a str,
}

struct Kiro<'a> {
    nodes: Vec<Node<'a>>,
    selected: Option<i32>,
}

// TODO: Custom fonts
// TODO: In-app config
// TODO: Better styling

impl<'a> Node<'a> {
    fn new(ids: &mut HashSet<i32>, hostname: &'a str, address: &'a str) -> Self {
        let mut rng = rand::rng();
        let mut id: i32;

        loop {
            id = rng.random_range(1..=9999);
            if !ids.contains(&id) {
                ids.insert(id);
                break;
            }
        }

        Self {
            id,
            hostname,
            address,
        }
    }
}

impl<'a> Kiro<'a> {
    fn new(nodes: Vec<Node<'a>>) -> Self {
        Self {
            nodes,
            selected: None,
        }
    }

    fn render_nodes(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            let available_height = ui.max_rect().height();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for node in &self.nodes {
                        let style = ui.style();

                        let mut layout_job = LayoutJob::default();

                        RichText::new(node.hostname).strong().size(14.0).append_to(
                            &mut layout_job,
                            &style,
                            FontSelection::Default,
                            Align::LEFT,
                        );

                        RichText::new(format!("\n{}", node.address))
                            .size(12.0)
                            .append_to(
                                &mut layout_job,
                                &style,
                                FontSelection::Default,
                                Align::LEFT,
                            );

                        let is_selected = self.selected == Some(node.id);
                        let button = Button::new(layout_job).selected(is_selected).wrap();

                        if ui.add(button).clicked() {
                            self.selected = Some(node.id);
                        }

                        ui.add_space(PADDING);
                    }
                });

                ui.scope(|ui| {
                    ui.set_min_height(available_height);
                    ui.add(Separator::default().vertical());
                });
            });
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
