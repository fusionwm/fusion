use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
};

use chrono::{DateTime, Local};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
pub struct Entry {
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
        return vec![];
        if log_file.exists() && log_file.metadata().is_ok_and(|m| m.len() > 0) {
            let mut file = std::fs::File::open(log_file).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes);

            let archived = unsafe { rkyv::archived_root::<Vec<Entry>>(&bytes) };
            archived.deserialize(&mut rkyv::Infallible).unwrap()
        } else {
            Vec::new()
        }
    }

    fn writer(log_file: PathBuf) -> BufWriter<File> {
        let file = std::fs::File::options()
            .append(true)
            .create(true)
            .open(log_file)
            .unwrap();
        BufWriter::new(file)
    }

    #[must_use]
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
        return;
        let bytes = rkyv::to_bytes::<_, 256>(entry).unwrap();
        self.writer.write_all(bytes.as_slice()).unwrap();
        self.writer.flush().unwrap();
    }
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::logging::Level::Debug, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::logging::Level::Info, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::logging::Level::Warn, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_format($crate::logging::Level::Error, format_args!($($arg)*))
    };
}
