// #![windows_subsystem = "windows"]
use std::time::Duration;

use eframe::{
    egui::{self, IconData, ViewportBuilder},
    epaint::{Color32, Vec2},
};

use notifications::{draw_notification, Notification};
use rodio::{Decoder, OutputStream, Sink};

mod precomputed;
mod timer;
use timer::PomodoroPhase;
mod ui;
mod utils;
use ui::Action;
use utils::Percent;
mod circle_widget;
mod notifications;
use circle_widget::ProgressCircle;
mod stats;
use stats::Stats;

#[cfg(windows)]
use utils::get_window_size;
use utils::MonitorSize;

const RAW_AUDIO: &[u8; 368684] = include_bytes!("./assets/completed.wav");

struct App {
    work_phase: Duration,
    break_phase: Duration,
    phase: PomodoroPhase,
    stats: Stats,
    notifications: Vec<Notification>,
    screen_size: MonitorSize,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let size = {
            let mut size = get_window_size().unwrap_or_default();
            size.width /= cc.egui_ctx.pixels_per_point();
            size.height /= cc.egui_ctx.pixels_per_point();
            size
        };
        App {
            work_phase: Duration::from_secs(30 * 60),
            break_phase: Duration::from_secs(15 * 60),
            phase: PomodoroPhase::new_work(Duration::from_secs(30 * 60)),
            stats: Stats::default(),
            notifications: vec![],
            screen_size: size,
        }
    }

    fn check_time(&mut self) {
        if let Some(start) = self.phase.get_start() {
            if start.elapsed() > self.phase.get_duration() {
                self.next_phase(false);
            }
        }
    }

    fn play_completed_sound() {
        std::thread::spawn(|| {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let my_slice = std::io::Cursor::new(RAW_AUDIO);
            let source = Decoder::new(my_slice).unwrap();
            sink.append(source);
            sink.sleep_until_end();
        });
    }

    fn next_phase(&mut self, skipped: bool) {
        Self::play_completed_sound();
        self.phase = match self.phase {
            PomodoroPhase::Work { .. } => {
                if !skipped {
                    self.notifications
                        .push(Notification::new("Work Done!".to_string()));
                }
                PomodoroPhase::new_break(self.break_phase)
            }
            PomodoroPhase::Break { .. } => {
                if !skipped {
                    self.notifications
                        .push(Notification::new("Break Done!".to_string()));
                }
                PomodoroPhase::new_work(self.work_phase)
            }
        };
        self.stats.increment();
    }
}

impl eframe::App for App {
    // This will get called every time the app updates, or every 5ms, which ever is faster
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // check if time is done
            self.check_time();

            // key inputs
            ui.input_mut(|i| {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Space) {
                    self.phase.toggle();
                }
            });

            // header
            ui::draw_header(ui);

            let percent = self
                .phase
                .to_percent()
                .unwrap_or(Percent::new(100).expect("Should be valid"));
            ui.add(ProgressCircle::new(percent, &mut self.phase));

            if let Action::NextPhase = ui::draw_stats_bar(ui, &self.stats) {
                self.next_phase(true);
            }

            if let Action::ModifyPhaseConfig {
                work_phase,
                break_phase,
            } = ui::draw_config(ui, &self.work_phase, &self.break_phase)
            {
                self.work_phase = work_phase;
                self.break_phase = break_phase;
            }
        });

        draw_notification(ctx, &mut self.notifications, &self.screen_size);

        // this is what sets the slowest update speed
        ctx.request_repaint_after(Duration::from_millis(5));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }
}

fn load_icon() -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(include_bytes!("./assets/icon.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn build_viewport() -> ViewportBuilder {
    ViewportBuilder::default()
        .with_resizable(false)
        .with_max_inner_size(Vec2::new(350.0, 450.0))
        .with_icon(load_icon())
}

fn main() {
    let native_options = eframe::NativeOptions {
        centered: true,
        viewport: build_viewport(),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Pomodoro Timer",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
