extern crate chrono;

use self::chrono::Local;
use std::fs::OpenOptions;
use std::io::prelude::*;

/// Enumaration to use different Log Levels
#[derive(Copy, Clone)]
pub enum LogLevel {
    OFF = 0,
    INFORMATION = 1,
    ERROR = 2,
    DEBUG = 3,
    VERBOSE = 4,
}

/// # Logger
///
/// This Structure is designed do provide
/// simple Logging Capabilities.
///
/// # TO-DO
/// - Make it thread safe
/// - Remove Unwrap etc.
pub struct Logger {
    pub logfile_path: String,
    pub log_level: LogLevel,
}

impl Logger {
    /// Create a new Instance of the Logger with the given
    /// LogLevel and Path to the File to create.
    pub fn new(log_path: &str, log_level: u8) -> Logger {

        let l_level = match log_level {
            0 => LogLevel::OFF,
            1 => LogLevel::INFORMATION,
            2 => LogLevel::ERROR,
            3 => LogLevel::DEBUG,
            _ => LogLevel::VERBOSE,
        };

        Logger {
            logfile_path: log_path.to_string(),
            log_level: l_level,
        }
    }

    /// Writes a new line containing the message to the log file.
    /// An Output will happen only if the log level of the message
    /// is inside of the one set in the logger.
    pub fn write_log(&self, message: &str, log_level: LogLevel) {
        if log_level as u8 <= self.log_level as u8 {
            let now = Local::now();

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.logfile_path)
                .unwrap();

            if let Err(_) = writeln!(file, "{}: {}", now.format("%d.%m.%Y %H:%M:%S.%f"), message) {
                println!("{}: {}", now.format("%d.%m.%Y %H:%M:%S.%f"), message);
            }
        }
    }
}
