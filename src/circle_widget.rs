use std::time::Duration;

use eframe::{
    egui::{self, Response, Sense, Widget},
    emath::Align2,
    epaint::{
        tessellator::{Path, PathType},
        Color32, FontFamily, FontId, Mesh, Pos2, Rect, Shape, Stroke, Vec2,
    },
};

use crate::{precomputed::CIRCLE, timer::PomodoroPhase, ui::Action, utils::Percent};

pub struct ProgressCircle<'a> {
    amount: Percent,
    phase: &'a mut PomodoroPhase,
}

impl<'a> ProgressCircle<'a> {
    pub fn new(p: Percent, phase: &'a mut PomodoroPhase) -> Self {
        Self { amount: p, phase }
    }

    fn get_points(&self, center: Pos2, radius: f32) -> Vec<Pos2> {
        let single: f32 = CIRCLE.len() as f32 / 360.0;
        let angle = 360.0 - self.amount.map_to_value(360.0);

        CIRCLE
            .iter()
            .enumerate()
            .skip_while(|(i, _)| *i as f64 / single as f64 <= angle)
            .map(|(_, &pos)| center + (radius * pos))
            .collect()
    }

    fn is_in_circle(&self, response: &Response, radius: f32, outer: Rect) -> bool {
        if let Some(hover_pos) = response.hover_pos() {
            let center = outer.center();
            ((center.x - hover_pos.x).abs().powf(2.0) + (center.y - hover_pos.y).abs().powf(2.0))
                .sqrt()
                <= radius + 7.5
        } else {
            false
        }
    }

    fn sense_hover(&self, ui: &mut egui::Ui, response: &Response, radius: f32, outer: Rect) {
        if response.hovered() && self.is_in_circle(response, radius, outer) {
            let mut color = self.phase.to_color(ui);
            color = color.gamma_multiply(0.5);
            ui.painter()
                .circle(outer.center(), radius, color, Stroke::NONE);
        }
    }

    fn sense_click(&self, response: &Response, radius: f32, outer: Rect) -> Action {
        if response.clicked() & self.is_in_circle(response, radius, outer) {
            return Action::TogglePhase;
        }
        Action::None
    }

    fn paint_progress_circle(&self, ui: &mut egui::Ui, radius: f32, outer: Rect) {
        let mut path = Path::default();
        let points = self.get_points(outer.center(), radius);
        if points.len() > 2 {
            // adds the points of the progress circle in the middle with the radius of ''
            path.add_open_points(&self.get_points(outer.center(), radius));
            // converts it to a mesh
            let mut mesh = Mesh::default();
            path.stroke(
                1.0,
                PathType::Open,
                Stroke::new(7.5, self.phase.to_color(ui)),
                &mut mesh,
            );
            // paints it
            ui.painter().add(Shape::Mesh(mesh));
        }
    }

    fn paint_info(&self, ui: &mut egui::Ui, outer: Rect) {
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
        let time_left = minutes + ":" + &seconds;

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
        self.sense_hover(ui, &response, radius, outer);

        // click effect
        if let Action::TogglePhase = self.sense_click(&response, radius, outer) {
            self.phase.toggle();
        }

        // paints the thin circle behind the progress one
        ui.painter().circle_stroke(
            outer.center(),
            radius,
            Stroke::new(1.5, ui.style().visuals.weak_text_color()),
        );

        self.paint_progress_circle(ui, radius, outer);

        self.paint_info(ui, outer);

        response
    }
}
