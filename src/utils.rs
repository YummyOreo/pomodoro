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
