use chrono::Local;
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
#[derive(Clone)]
pub struct Logger {
    pub logfile_path: String,
    pub log_level: LogLevel,
    pub daemonize: bool,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            logfile_path: String::from("/var/log/slms.log"),
            log_level: LogLevel::INFORMATION,
            daemonize: true,
        }
    }

    /// Create a new Instance of the Logger with the given
    /// LogLevel and Path to the File to create.
    pub fn set(&mut self, log_path: &str, log_level: u8, daemonize: bool) {

        let l_level = match log_level {
            0 => LogLevel::OFF,
            1 => LogLevel::INFORMATION,
            2 => LogLevel::ERROR,
            3 => LogLevel::DEBUG,
            _ => LogLevel::VERBOSE,
        };

        self.logfile_path = log_path.to_string();
        self.log_level = l_level;
        self.daemonize = daemonize;
    }

    /// Writes a new line containing the message to the log file.
    /// An Output will happen only if the log level of the message
    /// is inside of the one set in the logger.
    pub fn write_log(&self, message: &str, log_level: LogLevel) {
        if log_level as u8 <= self.log_level as u8 {
            let now = Local::now();

            if self.daemonize {
                let mut file = match OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&self.logfile_path) {
                    Ok(value) => value,
                    Err(_) => {
                        println!("{}: {}", now.format("%d.%m.%Y %H:%M:%S.%f"), message);
                        return;
                    }
                };

                match writeln!(file, "{}: {}", now.format("%d.%m.%Y %H:%M:%S.%f"), message) {
                    Err(_) => println!("{}: {}", now.format("%d.%m.%Y %H:%M:%S.%f"), message),
                    Ok(_) => {}
                }
            } else {
                println!("{}: {}", now.format("%d.%m.%Y %H:%M:%S.%f"), message);
            }
        }
    }
}
