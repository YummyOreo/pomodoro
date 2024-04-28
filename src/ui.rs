use std::time::Duration;

use eframe::egui::{Layout, ScrollArea, Slider, Ui};

use crate::{config::Status, stats::Stats};

pub enum Action {
    NextPhase,
    TogglePhase,
    None,
    ModifyWorkPhaseConfig(Duration),
    ModifyBreakPhaseConfig(Duration),
}

pub fn draw_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Pomodoro");
    });

    ui.separator();
}

pub fn draw_stats_bar(ui: &mut Ui, stats: &Stats, status: Status) -> Action {
    let mut action = Action::None;
    ui.horizontal(|ui| {
        let stats_text = format!(
            "{}/{}:{}",
            stats.get_phase_count().0,
            stats.get_phase_count().1,
            stats.get_count()
        );
        ui.label(stats_text);
            match status {
                Status::Saving => {
                    ui.label("Saving");
                    ui.spinner();
                }
                Status::Loading => {
                    ui.label("Loading");
                    ui.spinner();
                }
                Status::Saved => {
                    ui.label("Saved");
                }
                Status::Loaded => {
                    ui.label("Loaded");
                }
                _ => {}
            };
        ui.with_layout(Layout::top_down(eframe::emath::Align::Max), |ui| {
            if ui.button("Skip").clicked() {
                action = Action::NextPhase;
            }
        })
    });
    action
}

pub fn draw_config(ui: &mut Ui, work_phase: &Duration, break_phase: &Duration) -> Vec<Action> {
    ui.separator();
    let mut work_phase_new = *work_phase;
    let mut break_phase_new = *break_phase;
    ScrollArea::vertical().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let mut wtime = work_phase_new.as_secs() / 60;
        ui.horizontal(|ui| {
            ui.label("Work Time:");
            ui.add(Slider::new(&mut wtime, 0..=100).text("(min)"));
        });
        work_phase_new = Duration::from_secs(wtime * 60);

        let mut btime = break_phase_new.as_secs() / 60;
        ui.horizontal(|ui| {
            ui.label("Break Time:");
            ui.add(Slider::new(&mut btime, 0..=100).text("(min)"));
        });
        break_phase_new = Duration::from_secs(btime * 60);
    });
    let mut actions = vec![];
    if work_phase_new != *work_phase {
        actions.push(Action::ModifyWorkPhaseConfig(work_phase_new));
    }
    if break_phase_new != *break_phase {
        actions.push(Action::ModifyBreakPhaseConfig(break_phase_new));
    }
    actions
}
