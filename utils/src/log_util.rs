extern crate log;

use env_logger::fmt::style::{self, Color, RgbColor};

pub struct Log;
impl Log{
    pub fn init_log() {
        use chrono::Local;
        use std::io::Write;
    
        let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");
        env_logger::Builder::from_env(env)
            .format(|buf, record| {

                let style = match record.level(){
                    log::Level::Error => Color::Rgb(RgbColor(255, 0 ,0)).on_default(),
                    log::Level::Warn => Color::Rgb(RgbColor(255, 255 ,0)).on_default(),
                    log::Level::Debug | log::Level::Trace =>Color::Rgb(RgbColor(224, 255 ,255)).on_default(),
                    log::Level::Info => Color::Rgb(RgbColor(0, 255 ,127)).on_default().effects(style::Effects::BOLD),
                };
                
                writeln!(
                    buf,
                    "{} {style}{}{style:#} [{}:{}:{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.module_path().unwrap_or("<unnamed>"),
                    record.line().unwrap_or(0),
                    record.file().unwrap_or(""),
                    &record.args()
                )
            })
            .init();

    }
}

#[macro_export]
macro_rules! log_info {
     (target: $target:expr, $($arg:tt)+) => (log::info!(target: $target,$($arg)+));
     ($($arg:tt)+) => (log::info!($($arg)+))
}

#[macro_export]
macro_rules! log_debug {
     (target: $target:expr, $($arg:tt)+) => (log::debug!(target: $target,$($arg)+));
     ($($arg:tt)+) => (log::debug!($($arg)+));
}

#[macro_export]
macro_rules! log_error {
     (target: $target:expr, $($arg:tt)+) => (log::error!(target: $target,$($arg)+));
     ($($arg:tt)+) => (log::error!($($arg)+));
}

#[macro_export]
macro_rules! log_warn {
     (target: $target:expr, $($arg:tt)+) => (log::warn!(target: $target,$($arg)+));
     ($($arg:tt)+) => (log::warn!($($arg)+));
}