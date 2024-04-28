#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics;

#[derive(Debug, Default)]
pub struct MonitorSize {
    pub width: f32,
    pub height: f32,
}

pub fn get_window_size() -> Option<MonitorSize> {
    #[cfg(windows)]
    {
        let width = unsafe { GetSystemMetrics(0) } as f32;
        let height = unsafe { GetSystemMetrics(1) } as f32;

        return Some(MonitorSize { width, height });
    }
    None
}

pub struct Percent {
    percent: i8,
}

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
