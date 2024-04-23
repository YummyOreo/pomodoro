use std::time::{Duration, Instant};

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

struct Percent {
    n: i8,
}

impl Percent {
    pub fn new(n: i8) -> Result<Self, String> {
        // this percentage cannot go to 0 because then the indexing will break in the get_points
        // method
        if !(0..=100).contains(&n) {
            return Err("Out of bounds".to_owned());
        }
        Ok(Self { n })
    }

    pub fn get_percent(&self) -> i8 {
        self.n
    }
    pub fn set_percent(&mut self, n: i8) -> Result<(), String> {
        if !(0..=100).contains(&n) {
            return Err("Out of bounds".to_owned());
        }
        self.n = n;
        Ok(())
    }

    pub fn map_to_value(&self, value: usize) -> usize {
        let x: usize = ((self.n as f64 / 100.0) * (value as f64)) as usize;
        // clamps it because it could go over the value
        x.clamp(0, value)
    }
}

struct ProgressCircle {
    amount: Percent,
    color: egui::Color32,
}

impl ProgressCircle {
    pub fn new(p: Percent, phase: &PomodoroPhase) -> Self {
        Self {
            amount: p,
            color: phase.to_color(),
        }
    }

    fn get_points(&self, center: Pos2, radius: f32) -> Vec<Pos2> {
        // uses the precomputed values from the egui famework to make a partial circle
        let mut path = vec![];
        let offset = self
            .amount
            // maps the percentage to a value
            .map_to_value(CIRCLE_128.len() - 1);
        // gets all the points
        let quadrant_vertices: &[Vec2] = &CIRCLE_128[0..=offset];
        path.extend(quadrant_vertices.iter().map(|&n| center + radius * n));
        path
    }
}

impl Widget for ProgressCircle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // sets the amount of space the widget takes up
        let (outer, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), ui.available_height() / 1.5),
            Sense::focusable_noninteractive(),
        );
        // paints the thin circle behind the progress one
        ui.painter().circle_stroke(
            outer.center(),
            ui.available_width() / 3.5,
            Stroke::new(1.5, ui.style().visuals.weak_text_color()),
        );

        let mut path = Path::default();
        // adds the points of the progress circle in the middle with the radius of ''
        path.add_open_points(&self.get_points(outer.center(), ui.available_width() / 3.5));
        // converts it to a mesh
        let mut mesh = Mesh::default();
        path.stroke(1.0, PathType::Open, Stroke::new(7.5, self.color), &mut mesh);
        // paints it
        ui.painter().add(Shape::Mesh(mesh));
        // adds the text in the middle
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

enum PomodoroPhase {
    Work(Duration),
    Break(Duration),
}

impl PomodoroPhase {
    pub fn get_duration(&self) -> Duration {
        match self {
            Self::Work(d) | Self::Break(d) => *d,
        }
    }

    pub fn to_color(&self) -> Color32 {
        match self {
            Self::Work(_) => Color32::from_hex("#3aeb34").unwrap(),
            Self::Break(_) => Color32::from_hex("#dceb34").unwrap(),
        }
    }

    pub fn to_precent(&self, start: &Instant) -> Percent {
        println!(
            "{} {} {}",
            (start.elapsed().as_secs() as f64 / self.get_duration().as_secs() as f64) * 100.0,
            start.elapsed().as_secs(),
            self.get_duration().as_secs()
        );

        Percent::new(
            100 - ((start.elapsed().as_secs() as f64 / self.get_duration().as_secs() as f64)
                * 100.0)
                .clamp(0.0, i8::MAX as f64) as i8,
        )
        .unwrap()
    }
}

struct App {
    work_phase: Duration,
    break_phase: Duration,
    current_phase: Option<PomodoroPhase>,
    current_phase_start: Option<Instant>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        App {
            work_phase: Duration::from_secs(60),
            break_phase: Duration::from_secs(60),
            current_phase: Some(PomodoroPhase::Work(Duration::from_secs(1 * 60))),
            current_phase_start: Some(Instant::now()),
        }
    }

    fn check_time(&mut self) {
        if let Some(start) = self.current_phase_start {
            if let Some(phase) = &mut self.current_phase {
                if start.elapsed() > phase.get_duration() {
                    self.current_phase = Some(match phase {
                        PomodoroPhase::Work(_) => PomodoroPhase::Break(self.break_phase),
                        PomodoroPhase::Break(_) => PomodoroPhase::Work(self.break_phase),
                    });
                    self.current_phase_start = Some(Instant::now());
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_time();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Pomodoro");
            });
            ui.separator();
            ui.add(ProgressCircle::new(
                self.current_phase
                    .as_ref()
                    .unwrap()
                    .to_precent(self.current_phase_start.as_ref().unwrap()),
                self.current_phase.as_ref().unwrap(),
            ));
            ui.vertical_centered(|ui| {
                let _ = ui.button("Start");
            });
            ui.horizontal(|ui| {
                ui.label("1/2:0");
                ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
                    let _ = ui.button("Skip");
                })
            });
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                let mut wtime = self.work_phase.as_secs() / 60;
                ui.horizontal(|ui| {
                    ui.label("Work Time:");
                    ui.add(Slider::new(&mut wtime, 0..=100).text("(min)"));
                });
                self.work_phase = Duration::from_secs(wtime * 60);
                let mut btime = self.break_phase.as_secs() / 60;
                ui.horizontal(|ui| {
                    ui.label("Break Time:");
                    ui.add(Slider::new(&mut btime, 0..=100).text("(min)"));
                });
                self.break_phase = Duration::from_secs(btime * 60);
            });
        });
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

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
