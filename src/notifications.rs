use std::time::{Duration, Instant};

use eframe::{
    egui::{self, Sense, ViewportBuilder},
    emath::Align2,
    epaint::{vec2, Color32, FontFamily, FontId, Pos2, Stroke},
};

use crate::utils::MonitorSize;

const NOTIFICATION_DURATION: Duration = Duration::from_secs(5);

pub struct Notification {
    pub time_start: Instant,
    pub duration: Duration,
    pub text: String,
}

impl Notification {
    pub fn new(text: String) -> Self {
        Self {
            time_start: Instant::now(),
            duration: NOTIFICATION_DURATION,
            text,
        }
    }
}

fn notification_viewport(screen_size: &MonitorSize, i: f32) -> ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title("Immediate Viewport")
        .with_taskbar(false)
        .with_decorations(false)
        .with_position(Pos2::new(screen_size.width - 210.0, 10.0 + (i * 50.0)))
        .with_max_inner_size(vec2(200.0, 50.0))
        .with_always_on_top()
        .with_resizable(false)
}

fn draw_inner_notification(
    ctx: &egui::Context,
    notifications: &mut Vec<Notification>,
    i: usize,
    removed: &mut usize,
) {
    let frame = egui::Frame::none()
        .inner_margin(7.0)
        .stroke(Stroke::new(5.0, ctx.style().visuals.faint_bg_color))
        .fill(ctx.style().visuals.panel_fill);
    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        let (rect, sense) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        ui.painter().text(
            rect.left_center(),
            Align2::LEFT_CENTER,
            &notifications.get(i - *removed).expect("Should exist").text,
            FontId::new(20.0, FontFamily::default()),
            Color32::WHITE,
        );

        if sense.clicked() {
            notifications.remove(i - *removed);
            *removed += 1;
        }
    });
}

pub fn draw_notification(
    ctx: &egui::Context,
    notifications: &mut Vec<Notification>,
    screen_size: &MonitorSize,
) {
    #[cfg(windows)]
    {
        let mut removed = 0;
        for i in 0..notifications.len() {
            let notification = &notifications[i - removed];
            if notification.duration < notification.time_start.elapsed() {
                notifications.remove(i - removed);
                removed += 1;
                continue;
            }
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("immediate_viewport".to_owned() + &i.to_string()),
                notification_viewport(screen_size, i as f32),
                |ctx, _class| draw_inner_notification(ctx, notifications, i, &mut removed),
            );
        }
    }
}
