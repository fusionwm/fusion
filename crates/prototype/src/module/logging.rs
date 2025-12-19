use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use bincode::{Decode, Encode};
use chrono::{DateTime, Local};

#[derive(Debug, Encode, Decode)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Encode, Decode)]
pub struct Entry {
    #[bincode(with_serde)]
    time: DateTime<Local>,
    level: Level,
    message: String,
}

pub struct Logger {
    writer: BufWriter<File>,
    inner: Vec<Entry>,
}

impl Logger {
    fn try_read_binary_entries(log_file: PathBuf) -> Vec<Entry> {
        let mut entries = Vec::new();
        if log_file.exists() && log_file.metadata().is_ok_and(|m| m.len() > 0) {
            let mut file = std::fs::File::open(log_file).unwrap();
            let config = bincode::config::standard();
            while let Ok(entry) = bincode::decode_from_std_read(&mut file, config) {
                entries.push(entry);
            }
        }
        entries
    }

    fn writer(log_file: PathBuf) -> BufWriter<File> {
        let file = std::fs::File::options()
            .append(true)
            .create(true)
            .open(log_file)
            .unwrap();
        BufWriter::new(file)
    }

    pub fn new(log_file: PathBuf) -> Self {
        Self {
            inner: Self::try_read_binary_entries(log_file.clone()),
            writer: Self::writer(log_file),
        }
    }

    pub fn log_format(&mut self, level: Level, args: std::fmt::Arguments) {
        let entry = Entry {
            time: Local::now(),
            level,
            message: args.to_string(),
        };
        self.write(&entry);
        self.inner.push(entry);
    }

    fn write(&mut self, entry: &Entry) {
        let config = bincode::config::standard();
        bincode::encode_into_std_write(entry, &mut self.writer, config).unwrap();
        self.writer.flush().unwrap();
    }
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::module::logging::Level::Debug, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::module::logging::Level::Info, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::module::logging::Level::Warn, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::module::logging::Level::Error, format_args!($($arg)*))
    };
}
