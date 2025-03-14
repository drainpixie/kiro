use crate::constants::{PADDING, SECONDS_PER_HOUR};
use crate::websocket::WebSocketManager;
use eframe::App;
use egui::{
    menu, text::LayoutJob, Align, Button, CentralPanel, Context, FontSelection, Label, Layout,
    RichText, ScrollArea, Separator, TopBottomPanel,
};
use egui_plot::{AxisHints, GridMark, Legend, Line, Plot, PlotPoints};
use humantime::format_duration;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

pub struct Node<'a> {
    pub id: i32,

    pub uptime: Duration,

    pub os: &'a str,
    pub kernel: &'a str,
    pub address: &'a str,
    pub hostname: &'a str,
}

pub struct Kiro<'a> {
    pub nodes: &'a Vec<Node<'a>>,
    pub selected: Option<i32>,

    pub start: SystemTime,
    pub ram_history: HashMap<i32, Vec<(f64, f64)>>,

    manager: WebSocketManager<'a>,
}

impl<'a> Node<'a> {
    pub fn new(ids: &mut HashSet<i32>, hostname: &'a str, address: &'a str) -> Self {
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

            // Mock data:
            os: "NixOS 24.05",
            kernel: "6.8.0-48-generic",
            uptime: Duration::from_secs(3600 * 24 * 3 + 3600 * 5 + 600),
        }
    }
}

impl<'a> Kiro<'a> {
    pub fn new(nodes: &'a Vec<Node<'a>>) -> Self {
        let kiro = Self {
            nodes,
            selected: None,
            start: SystemTime::now(),
            ram_history: HashMap::new(),
            manager: WebSocketManager::new(),
        };

        for node in nodes {
            kiro.manager.connect(node.hostname, node.address);
        }

        kiro
    }

    pub fn mock_data(&mut self) {
        let mut rng = rand::rng();

        let elapsed = SystemTime::now()
            .duration_since(self.start)
            .expect("system clock error");

        let elapsed_hours = elapsed.as_secs_f64() / SECONDS_PER_HOUR;

        for node in self.nodes {
            let history = self.ram_history.entry(node.id).or_default();
            let last_value = history.last().map(|(_, usage)| *usage).unwrap_or(50.0);
            let new_value = (last_value + rng.random_range(-2.0..2.0)).clamp(30.0, 90.0);

            history.push((elapsed_hours, new_value));
            history.retain(|(t, _)| elapsed_hours - *t <= 24.0);
        }
    }

    pub fn render_nodes(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            let available_height = ui.max_rect().height();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for node in self.nodes {
                        let style = ui.style();

                        let mut layout_job = LayoutJob::default();

                        RichText::new(node.hostname).strong().size(14.0).append_to(
                            &mut layout_job,
                            style,
                            FontSelection::Default,
                            Align::LEFT,
                        );

                        RichText::new(format!("\n{}", node.address))
                            .size(12.0)
                            .append_to(&mut layout_job, style, FontSelection::Default, Align::LEFT);

                        let is_selected = self.selected == Some(node.id);
                        let button = Button::new(layout_job)
                            .selected(is_selected)
                            .min_size(egui::vec2(50.0, 20.0))
                            .wrap();

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

    pub fn render_ram_history(&self, node: &Node, ui: &mut egui::Ui) {
        let history = self.ram_history.get(&node.id).expect("history not found");

        let points: PlotPoints = history
            .iter()
            .map(|(seconds, usage)| [*seconds, *usage])
            .collect::<Vec<_>>()
            .into();

        let line = Line::new(points).name("RAM Usage (%)");

        let percentage_axis = AxisHints::new_y().label("RAM Usage (%)");
        let custom_time_axis = AxisHints::new_x().label("Time").formatter(Box::new(
            |mark: GridMark, _range: &std::ops::RangeInclusive<f64>| {
                let seconds = mark.value * SECONDS_PER_HOUR;
                let duration = Duration::from_secs(seconds as u64);
                format_duration(duration).to_string()
            },
        ));

        Plot::new("ram_history")
            .view_aspect(4.0)
            .legend(Legend::default())
            .custom_x_axes(vec![custom_time_axis])
            .custom_y_axes(vec![percentage_axis])
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }

    pub fn create_info_column(ui: &mut egui::Ui, title: &str, value: &str) {
        let mut job = LayoutJob::default();
        let style = ui.style();

        RichText::new(title).heading().append_to(
            &mut job,
            style,
            FontSelection::Default,
            Align::LEFT,
        );

        RichText::new(format!("\n{}", value)).append_to(
            &mut job,
            style,
            FontSelection::Default,
            Align::LEFT,
        );

        ui.label(job);
        ui.add_space(PADDING);
    }

    pub fn render_node(&self, node: &Node, ui: &mut egui::Ui) {
        ui.add_space(PADDING);
        ui.label(RichText::new(node.hostname).heading());
        ui.add_space(PADDING);

        ui.horizontal(|ui| {
            Kiro::create_info_column(ui, "OS", node.os);
            Kiro::create_info_column(ui, "Kernel", node.kernel);
            Kiro::create_info_column(ui, "Uptime", &format_duration(node.uptime).to_string());
            Kiro::create_info_column(ui, "Address", node.address);
        });

        ui.add_space(PADDING);
    }

    pub fn render_menu(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("kiro_top_panel").show(ctx, |ui| {
            ui.add_space(PADDING);

            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.add(Label::new(RichText::new("📊 Kiro").heading()))
                });

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    ui.add(Label::new(RichText::new("🛠").heading()))
                })
            });

            ui.add_space(PADDING);
        });
    }
}

impl App for Kiro<'_> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.render_menu(ctx, frame);

        CentralPanel::default().show(ctx, |ui| {
            let available_height = ui.max_rect().height();
            self.mock_data();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_height(available_height);
                    self.render_nodes(ui);
                });

                ui.vertical(|ui| {
                    ui.set_min_height(available_height);

                    if let Some(id) = self.selected {
                        let node = self
                            .nodes
                            .iter()
                            .find(|&node| node.id == id)
                            .expect("couldn't find node");

                        ui.vertical_centered(|ui| {
                            self.render_node(node, ui);
                            self.render_ram_history(node, ui);
                        });
                    } else {
                        ui.label("Select a node for its data.");
                    }
                });
            });
        });
    }
}
