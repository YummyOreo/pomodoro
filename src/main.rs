#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

use config::ConfigManager;
use eframe::{
    egui::{self, ViewportBuilder},
    epaint::{Color32, Vec2},
    Theme,
};

use notifications::{draw_notification, Notification};

mod precomputed;
mod timer;
use timer::PomodoroPhase;
mod ui;
mod utils;
use ui::Action;
use utils::{play_sound, Percent};
mod circle_widget;
mod notifications;
use circle_widget::ProgressCircle;
mod stats;
use stats::Stats;
mod config;

#[cfg(windows)]
use utils::get_window_size;
use utils::MonitorSize;

const RAW_COMPLETE_SOUND: &[u8; 368684] = include_bytes!("./assets/completed.wav");

struct App {
    phase: PomodoroPhase,
    stats: Stats,
    notifications: Vec<Notification>,
    screen_size: MonitorSize,
    config_manager: ConfigManager,
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
        let mut config_manager = ConfigManager::new();
        config_manager.load();
        App {
            phase: PomodoroPhase::new_work(config_manager.get_work_time()),
            stats: Stats::default(),
            notifications: vec![],
            screen_size: size,
            config_manager,
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
        play_sound(RAW_COMPLETE_SOUND)
    }

    fn next_phase(&mut self, skipped: bool) {
        Self::play_completed_sound();
        self.phase = match self.phase {
            PomodoroPhase::Work { .. } => {
                if !skipped {
                    self.notifications
                        .push(Notification::new("Work Done!".to_string()));
                }
                PomodoroPhase::new_break(self.config_manager.get_break_time())
            }
            PomodoroPhase::Break { .. } => {
                if !skipped {
                    self.notifications
                        .push(Notification::new("Break Done!".to_string()));
                }
                PomodoroPhase::new_work(self.config_manager.get_work_time())
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
                .unwrap_or(Percent::new(100.0).expect("Should be valid"));
            ui.add(ProgressCircle::new(percent, &mut self.phase));

            let status = *self.config_manager.status.lock().unwrap();
            if let Action::NextPhase = ui::draw_stats_bar(ui, &self.stats, status) {
                self.next_phase(true);
            }

            let config_actions = ui::draw_config(
                ui,
                &self.config_manager.get_work_time(),
                &self.config_manager.get_break_time(),
            );
            for action in &config_actions {
                match action {
                    Action::ModifyWorkPhaseConfig(d) => self.config_manager.set_work_time(*d),
                    Action::ModifyBreakPhaseConfig(d) => self.config_manager.set_break_time(*d),
                    _ => {}
                }
            }
            if !config_actions.is_empty() {
                self.config_manager.save();
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

fn main() {
    fn build_viewport() -> ViewportBuilder {
        ViewportBuilder::default()
            .with_resizable(false)
            .with_max_inner_size(Vec2::new(350.0, 450.0))
            .with_icon(utils::load_icon())
    }

    let native_options = eframe::NativeOptions {
        centered: true,
        viewport: build_viewport(),
        default_theme: Theme::Dark,
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Pomodoro Timer",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
