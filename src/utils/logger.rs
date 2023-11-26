extern crate log;
use crate::utils::atomic::{self, SimpleAtomic};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, OnceLock},
};
pub(crate) struct Logger {
    pub enabled: atomic::Bool,
    pub data: Arc<Mutex<VecDeque<(String, log::Level)>>>,
}
impl Logger {
    pub fn new() -> Self {
        let data = Arc::new(Mutex::new(VecDeque::new()));

        Self {
            enabled: atomic::Bool::new(true),
            data,
        }
    }
    pub fn get_color(level: log::Level) -> egui::Color32 {
        match level {
            log::Level::Error => egui::Color32::RED,
            log::Level::Warn => egui::Color32::YELLOW,
            log::Level::Info => egui::Color32::WHITE,
            log::Level::Debug => egui::Color32::DEBUG_COLOR,
            log::Level::Trace => egui::Color32::BLUE,
        }
    }
}
impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.enabled.load() && metadata.level() <= log::STATIC_MAX_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if let Ok(mut txt) = self.data.try_lock() {
            let t = format!(
                "{}:{} -- {}",
                record.level(),
                record.target(),
                record.args()
            );
            println!("{}", t);
            txt.push_front((t, record.level()));
            txt.truncate(1000);
        }
    }

    fn flush(&self) {
        if let Ok(mut txt) = self.data.try_lock() {
            txt.clear();
        }
    }
}
pub(crate) static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();
