use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Debug, Copy, Clone)]
pub enum Status {
    Saving,
    Saved,
    Loading,
    Loaded,
    None,
}

pub struct ConfigManager {
    pub status: Arc<Mutex<Status>>,
    pub config: Arc<Mutex<Config>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(Status::None)),
            config: Arc::new(Mutex::new(Config::default())),
        }
    }

    fn get_save_dir() -> Option<PathBuf> {
        BaseDirs::new().map(|d| d.config_dir().to_path_buf())
    }

    pub fn load(&mut self) {
        let (status, config, mut config_file) = (
            self.status.clone(),
            self.config.clone(),
            Self::get_save_dir().unwrap(),
        );
        std::thread::spawn(move || {
            println!("Loading");
            *status.lock().unwrap() = Status::Loading;
            config_file.push("Pomodoro/config.toml");
            if config_file.exists() {
                if let Ok(data) = std::fs::read_to_string(config_file) {
                    *config.lock().unwrap() = toml::from_str(&data).unwrap();
                }
            }
            *status.lock().unwrap() = Status::Loaded;
            println!("Loaded");
        });
    }

    pub fn save(&mut self) {
        let (status, config, mut storage_dir) = (
            self.status.clone(),
            self.config.clone(),
            Self::get_save_dir().unwrap(),
        );
        std::thread::spawn(move || {
            println!("Saving");
            while !matches!(*status.lock().unwrap(), Status::Saved | Status::None | Status::Loaded) {
                std::thread::sleep(Duration::from_nanos(500));
            }
            *status.lock().unwrap() = Status::Saving;
            storage_dir.push("Pomodoro");
            if !storage_dir.exists() {
                let _ = std::fs::create_dir_all(&storage_dir);
            }
            storage_dir.push("config.toml");
            let _ = std::fs::write(
                storage_dir,
                toml::to_string(&*config.lock().unwrap()).unwrap(),
            );
            *status.lock().unwrap() = Status::Saved;
            println!("Saved");
        });
    }

    pub fn get_break_time(&self) -> Duration {
        self.config.lock().unwrap().break_time
    }
    pub fn get_work_time(&self) -> Duration {
        self.config.lock().unwrap().work_time
    }
    pub fn set_work_time(&self, time: Duration) {
        self.config.lock().unwrap().work_time = time;
    }
    pub fn set_break_time(&self, time: Duration) {
        self.config.lock().unwrap().break_time = time;
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub work_time: Duration,
    pub break_time: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_time: Duration::from_secs(30 * 60),
            break_time: Duration::from_secs(15 * 60),
        }
    }
}
