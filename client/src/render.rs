use crate::{constants::PADDING, nodes::Node};
use eframe::App;
use egui::{
    text::LayoutJob, Align, Button, CentralPanel, FontSelection, RichText, ScrollArea, Separator,
};

pub struct Kiro<'a> {
    nodes: Vec<Node<'a>>,
    selected: Option<i32>,
}

impl<'a> Kiro<'a> {
    pub fn new(nodes: Vec<Node<'a>>) -> Self {
        Self {
            nodes,
            selected: None,
        }
    }

    pub fn render_nodes(&mut self, ui: &mut egui::Ui) {
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
