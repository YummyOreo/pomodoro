use std::time::{Duration, Instant};

use eframe::{
    egui::{
        self, Frame, Layout, Response, RichText, ScrollArea, Sense, Slider, ViewportBuilder, Widget,
    },
    emath::{lerp, Align2},
    epaint::{
        tessellator::{Path, PathType},
        vec2, Color32, FontFamily, FontId, Mesh, Pos2, Rect, Rgba, Rounding, Shape, Stroke, Vec2,
    },
    WindowBuilder,
};

use rodio::{source::Source, Decoder, OutputStream, Sink};

use precomputed::CIRCLE;
mod precomputed;

struct Percent {
    percent: i8,
}

const RAW_AUDIO: &[u8; 368684] = include_bytes!("./assets/completed.wav");

impl Percent {
    pub fn new(n: i8) -> Result<Self, String> {
        // this percentage cannot go to 0 because then the indexing will break in the get_points
        // method
        if !(0..=100).contains(&n) {
            return Err("Out of bounds".to_owned());
        }
        Ok(Self { percent: n })
    }

    pub fn map_to_value(&self, value: usize) -> usize {
        let x: usize = ((self.percent as f64 / 100.0) * (value as f64)) as usize;
        // clamps it because it could go over the value
        x.clamp(0, value)
    }
}

struct ProgressCircle<'a> {
    amount: Percent,
    color: egui::Color32,
    phase: &'a mut PomodoroPhase,
}

impl<'a> ProgressCircle<'a> {
    pub fn new(p: Percent, phase: &'a mut PomodoroPhase) -> Self {
        Self {
            amount: p,
            color: phase.to_color(),
            phase,
        }
    }

    fn get_points(&self, center: Pos2, radius: f32) -> Vec<Pos2> {
        // uses the precomputed values from the egui famework to make a partial circle
        let mut path = vec![];
        let offset = self
            .amount
            // maps the percentage to a value
            .map_to_value(CIRCLE.len() - 1);
        // gets all the points
        let quadrant_vertices: &[Vec2] = &CIRCLE[0..=offset];
        path.extend(quadrant_vertices.iter().map(|&n| center + radius * n));
        path
    }
}

impl<'a> Widget for ProgressCircle<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let radius = ui.available_width() / 3.5;
        // sets the amount of space the widget takes up
        let (outer, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), ui.available_height() / 1.5),
            Sense::click(),
        );
        // hover effect
        if response.hovered() {
            if let Some(hover_pos) = response.hover_pos() {
                let center = outer.center();
                if ((center.x - hover_pos.x).abs().powf(2.0)
                    + (center.y - hover_pos.y).abs().powf(2.0))
                .sqrt()
                    <= radius + 7.5
                {
                    let mut color = ui.style().visuals.gray_out(self.color);
                    color = color.gamma_multiply(0.5);
                    ui.painter().circle(
                        outer.center(),
                        radius,
                        color,
                        Stroke::new(0.0, Color32::TRANSPARENT),
                    );
                }
            }
        }

        // click effect
        if response.clicked() {
            if let Some(hover_pos) = response.hover_pos() {
                let center = outer.center();
                if ((center.x - hover_pos.x).abs().powf(2.0)
                    + (center.y - hover_pos.y).abs().powf(2.0))
                .sqrt()
                    <= radius + 7.5
                {
                    match self.phase.get_start() {
                        Some(_) => {
                            self.phase.pause();
                        }
                        None => {
                            self.phase.start();
                        }
                    }
                }
            }
        }

        // paints the thin circle behind the progress one
        ui.painter().circle_stroke(
            outer.center(),
            radius,
            Stroke::new(1.5, ui.style().visuals.weak_text_color()),
        );

        let mut path = Path::default();
        // adds the points of the progress circle in the middle with the radius of ''
        path.add_open_points(&self.get_points(outer.center(), radius));
        // converts it to a mesh
        let mut mesh = Mesh::default();
        path.stroke(1.0, PathType::Open, Stroke::new(7.5, self.color), &mut mesh);
        // paints it
        ui.painter().add(Shape::Mesh(mesh));
        let time_left = {
            let seconds_left = self.phase.get_duration().as_secs()
                - self
                    .phase
                    .time_elapsed()
                    .unwrap_or(Duration::new(0, 0))
                    .as_secs();
            let mut minutes = (seconds_left / 60).to_string();
            let mut seconds = (seconds_left % 60).to_string();
            if minutes.len() < 2 {
                minutes = "0".to_owned() + &minutes;
            }
            if seconds.len() < 2 {
                seconds = "0".to_owned() + &seconds;
            }
            minutes + ":" + &seconds
        };
        // adds the text in the middle
        ui.painter().text(
            outer.center(),
            Align2::CENTER_CENTER,
            time_left,
            FontId::new(50.0, FontFamily::default()),
            Color32::WHITE,
        );
        let phase_text_placement = Pos2::new(outer.center().x, outer.center().y + 50.0);
        let phase_text = match self.phase {
            PomodoroPhase::Work { .. } => "Work",
            PomodoroPhase::Break { .. } => "Break",
        };
        ui.painter().text(
            phase_text_placement,
            Align2::CENTER_CENTER,
            phase_text,
            FontId::new(25.0, FontFamily::default()),
            Color32::WHITE,
        );
        response
    }
}

enum PomodoroPhase {
    Work {
        start: Option<Instant>,
        length: Duration,
        paused: Option<Duration>,
    },
    Break {
        start: Option<Instant>,
        length: Duration,
        paused: Option<Duration>,
    },
}

impl PomodoroPhase {
    pub fn new_work(length: Duration) -> Self {
        Self::Work {
            start: None,
            length,
            paused: Some(Duration::new(0, 0)),
        }
    }

    pub fn new_break(length: Duration) -> Self {
        Self::Break {
            start: None,
            length,
            paused: Some(Duration::new(0, 0)),
        }
    }

    pub fn get_duration(&self) -> Duration {
        match self {
            Self::Work { length, .. } | Self::Break { length, .. } => *length,
        }
    }

    pub fn get_start(&self) -> Option<Instant> {
        match self {
            Self::Work { start, .. } | Self::Break { start, .. } => *start,
        }
    }

    pub fn start(&mut self) {
        let sub = match self {
            Self::Work { paused, .. } | Self::Break { paused, .. } => paused.take(),
        }
        .unwrap_or(Duration::new(0, 0));
        match self {
            Self::Work { start, .. } | Self::Break { start, .. } => {
                *start = Some(Instant::now() - sub)
            }
        }
    }

    pub fn pause(&mut self) {
        match self {
            Self::Work { start, paused, .. } | Self::Break { start, paused, .. } => {
                if let Some(start_time) = start.take() {
                    *paused = Some(start_time.elapsed());
                }
            }
        }
    }

    pub fn is_paused(&self) -> bool {
        match self {
            Self::Break { paused, .. } | Self::Work { paused, .. } => paused.is_some(),
        }
    }

    pub fn to_color(&self) -> Color32 {
        if self.is_paused() {
            return Color32::from_hex("#34eb9f").unwrap();
        }
        match self {
            Self::Work { .. } => Color32::from_hex("#3aeb34").unwrap(),
            Self::Break { .. } => Color32::from_hex("#dceb34").unwrap(),
        }
    }

    pub fn time_elapsed(&self) -> Option<Duration> {
        match self {
            Self::Work { paused, start, .. } | Self::Break { paused, start, .. } => {
                if let Some(paused_time) = *paused {
                    Some(paused_time)
                } else {
                    start.map(|start_time| start_time.elapsed())
                }
            }
        }
    }

    pub fn to_precent(&self) -> Option<Percent> {
        if let Some(time_elapsed) = self.time_elapsed() {
            return Some(
                Percent::new(
                    100 - ((time_elapsed.as_secs() as f64 / self.get_duration().as_secs() as f64)
                        * 100.0)
                        .clamp(0.0, i8::MAX as f64) as i8,
                )
                .expect("Should be valid"),
            );
        }
        None
    }
}

#[derive(Default)]
struct Stats(pub usize);

impl Stats {
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn get_phase_count(&self) -> (i8, i8) {
        ((self.0 % 2 + 1) as i8, 2)
    }
    pub fn get_count(&self) -> usize {
        self.0 / 2
    }
}

struct App {
    work_phase: Duration,
    break_phase: Duration,
    phase: PomodoroPhase,
    stats: Stats,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        App {
            work_phase: Duration::from_secs(30 * 60),
            break_phase: Duration::from_secs(15 * 60),
            phase: PomodoroPhase::new_work(Duration::from_secs(30 * 60)),
            stats: Stats::default(),
        }
    }

    fn check_time(&mut self) {
        if let Some(start) = self.phase.get_start() {
            if start.elapsed() > self.phase.get_duration() {
                self.next();
            }
        }
    }

    fn play_completed() {
        std::thread::spawn(|| {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let my_slice = std::io::Cursor::new(RAW_AUDIO);
            let source = Decoder::new(my_slice).unwrap();
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    fn next(&mut self) {
        Self::play_completed();
        self.phase = match self.phase {
            PomodoroPhase::Work { .. } => PomodoroPhase::new_break(self.break_phase),
            PomodoroPhase::Break { .. } => PomodoroPhase::new_work(self.work_phase),
        };
        self.stats.increment();
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
            let percent = self
                .phase
                .to_precent()
                .unwrap_or(Percent::new(100).expect("Should be valid"));
            ui.add(ProgressCircle::new(percent, &mut self.phase));
            ui.horizontal(|ui| {
                let stats_text = format!(
                    "{}/{}:{}",
                    self.stats.get_phase_count().0,
                    self.stats.get_phase_count().1,
                    self.stats.get_count()
                );
                ui.label(stats_text);
                ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
                    if ui.button("Skip").clicked() {
                        self.next();
                    }
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
        ctx.request_repaint_after(Duration::from_millis(5));
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
