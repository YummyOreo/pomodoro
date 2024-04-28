use eframe::egui::IconData;
use rodio::{Decoder, OutputStream, Sink};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics;

#[derive(Debug, Default)]
pub struct MonitorSize {
    pub width: f32,
    pub height: f32,
}

#[allow(unreachable_code)]
pub fn get_window_size() -> Option<MonitorSize> {
    #[cfg(windows)]
    {
        let width = unsafe { GetSystemMetrics(0) } as f32;
        let height = unsafe { GetSystemMetrics(1) } as f32;

        return Some(MonitorSize { width, height });
    }
    None
}

pub fn play_sound(sound: &'static [u8]) {
    std::thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let my_slice = std::io::Cursor::new(sound);
        let source = Decoder::new(my_slice).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    });
}

pub fn load_icon() -> IconData {
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

pub struct Percent {
    percent: f32,
}

impl Percent {
    pub fn new(n: f32) -> Result<Self, String> {
        // this percentage cannot go to 0 because then the indexing will break in the get_points
        // method
        if !(0.0..=100.0).contains(&n) {
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
