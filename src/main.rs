use eframe::{
    egui::{self, Frame, Layout, ScrollArea, Sense, Slider, ViewportBuilder, Widget},
    emath::Align2,
    epaint::{
        tessellator::{Path, PathType},
        vec2, Color32, FontFamily, FontId, Mesh, Pos2, Rect, Rounding, Shape, Stroke, Vec2,
    },
    WindowBuilder,
};
use precomputed::CIRCLE_128;
mod precomputed;

fn build_viewport(builder: ViewportBuilder) -> ViewportBuilder {
    builder
        .with_resizable(false)
        .with_max_inner_size(Vec2::new(350.0, 450.0))
}

fn main() {
    let native_options = eframe::NativeOptions {
        centered: true,
        window_builder: Some(Box::new(build_viewport)),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Pomodoro Timer",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}

struct Percent {
    n: i8,
}

impl Percent {
    pub fn new(n: i8) -> Result<Self, String> {
        if !(1..=100).contains(&n) {
            return Err("Out of bounds".to_owned());
        }
        Ok(Self { n })
    }

    pub fn get_percent(&self) -> i8 {
        self.n
    }
    pub fn set_percent(&mut self, n: i8) -> Result<(), String> {
        if !(1..=100).contains(&n) {
            return Err("Out of bounds".to_owned());
        }
        self.n = n;
        Ok(())
    }

    pub fn map_to_n(&self, n: usize) -> usize {
        let x: usize = ((self.n as f64 / 100.0) * (n as f64)) as usize;
        x.clamp(0, n - 1)
    }
}

struct ProgressCircle {
    amount: Percent,
    color: egui::Color32,
}

impl ProgressCircle {
    pub fn new(p: Percent) -> Self {
        Self {
            amount: p,
            color: Color32::from_hex("#3aeb34").unwrap(),
        }
    }

    fn get_points(&self, center: Pos2, radius: f32) -> Vec<Pos2> {
        let mut path = vec![];
        let offset = self
            .amount
            .map_to_n(CIRCLE_128.len());
        let quadrant_vertices: &[Vec2] = &CIRCLE_128[0..=offset];
        path.extend(quadrant_vertices.iter().map(|&n| center + radius * n));
        path
    }
}

impl Widget for ProgressCircle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (outer, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), ui.available_height() / 1.5),
            Sense::hover(),
        );
        ui.painter().circle_stroke(outer.center(), ui.available_width() / 3.5, Stroke::new(1.5, ui.style().visuals.weak_text_color()));

        let mut path = Path::default();
        path.add_open_points(&self.get_points(outer.center(), ui.available_width() / 3.5));
        let mut mesh = Mesh::default();
        path.stroke(1.0, PathType::Open, Stroke::new(7.5, self.color), &mut mesh);
        ui.painter().add(Shape::Mesh(mesh));
        ui.painter().text(
            outer.center(),
            Align2::CENTER_CENTER,
            "30min",
            FontId::new(50.0, FontFamily::default()),
            Color32::WHITE,
        );
        response
    }
}

#[derive(Default)]
struct App {
    work_time: f32,
    break_time_1: f32,
    break_time_2: f32,
    t: i8,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        App {
            work_time: 30.0,
            break_time_1: 10.0,
            break_time_2: 15.0,
            t: 100,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Pomodoro");
            });
            ui.separator();
            ui.add(ProgressCircle::new(Percent::new(self.t).unwrap()));
            ui.vertical_centered(|ui| {
                let _ = ui.button("Start");
            });
            ui.horizontal(|ui| {
                ui.label("1/2:0");
                ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
                    let _ = ui.button("Skip");
                })
            });
            ui.add(Slider::new(&mut self.t, 1..=100).text("(min)"));
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label("Work Time:");
                    ui.add(Slider::new(&mut self.work_time, 0.0..=100.0).text("(min)"));
                });
                ui.horizontal(|ui| {
                    ui.label("Break Time (1):");
                    ui.add(Slider::new(&mut self.break_time_1, 0.0..=100.0).text("(min)"));
                });
                ui.horizontal(|ui| {
                    ui.label("Break Time (2):");
                    ui.add(Slider::new(&mut self.break_time_2, 0.0..=100.0).text("(min)"));
                });
            });
        });
    }
}
