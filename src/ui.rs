use std::time::Duration;

use eframe::egui::{Layout, ScrollArea, Slider, Ui};

use crate::stats::Stats;

pub enum Action {
    NextPhase,
    TogglePhase,
    None,
    ModifyPhaseConfig {
        work_phase: Duration,
        break_phase: Duration,
    },
}

pub fn draw_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Pomodoro");
    });
    ui.separator();
}

pub fn draw_stats_bar(ui: &mut Ui, stats: &Stats) -> Action {
    let mut action = Action::None;
    ui.horizontal(|ui| {
        let stats_text = format!(
            "{}/{}:{}",
            stats.get_phase_count().0,
            stats.get_phase_count().1,
            stats.get_count()
        );
        ui.label(stats_text);
        ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
            if ui.button("Skip").clicked() {
                action = Action::NextPhase;
            }
        })
    });
    ui.separator();
    action
}

pub fn draw_config(ui: &mut Ui, work_phase: &Duration, break_phase: &Duration) -> Action {
    let mut work_phase = *work_phase;
    let mut break_phase = *break_phase;
    ScrollArea::vertical().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let mut wtime = work_phase.as_secs() / 60;
        ui.horizontal(|ui| {
            ui.label("Work Time:");
            ui.add(Slider::new(&mut wtime, 0..=100).text("(min)"));
        });
        work_phase = Duration::from_secs(wtime * 60);
        let mut btime = break_phase.as_secs() / 60;
        ui.horizontal(|ui| {
            ui.label("Break Time:");
            ui.add(Slider::new(&mut btime, 0..=100).text("(min)"));
        });
        break_phase = Duration::from_secs(btime * 60);
    });
    Action::ModifyPhaseConfig {
        work_phase,
        break_phase,
    }
}
