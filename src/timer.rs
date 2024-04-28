use std::time::{Duration, Instant};

use eframe::{egui, epaint::Color32};

use crate::utils::Percent;

pub enum PomodoroPhase {
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

    pub fn toggle(&mut self) {
        if self.is_paused() {
            self.start();
        } else {
            self.pause();
        }
    }

    pub fn to_color(&self, ui: &mut egui::Ui) -> Color32 {
        let color = match self {
            Self::Work { .. } => Color32::from_hex("#3aeb34").unwrap(),
            Self::Break { .. } => Color32::from_hex("#dceb34").unwrap(),
        };
        if self.is_paused() {
            ui.style().visuals.gray_out(color)
        } else {
            color
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

    pub fn to_percent(&self) -> Option<Percent> {
        if let Some(time_elapsed) = self.time_elapsed() {
            return Some(
                Percent::new(
                    100.0
                        - ((time_elapsed.as_secs() as f64 / self.get_duration().as_secs() as f64)
                            * 100.0)
                            .clamp(0.0, 100.0) as f32,
                )
                .expect("Should be valid"),
            );
        }
        None
    }
}
