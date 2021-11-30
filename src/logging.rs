//! if there's a log file, write to it.

use crate::config::Config;

use std::fs;
use std::io::Write;

extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Local;
use std::time::{Duration, UNIX_EPOCH};

/// Write errors to log, or to stderr if that fails.
pub fn log(conf: &Config, message: &str) {
    write_log(conf, message, true).expect("Failed writing to log file");
}

/// Write successful outcomes to log only, don't put on stderr.
pub fn log_only(conf: &Config, message: &str) {
    write_log(conf, message, false).expect("Failed writing to log file");
}

fn write_log(conf: &Config, message: &str, always: bool) -> std::io::Result<()> {
    let d = UNIX_EPOCH + Duration::from_secs(conf.current_time);
    let datetime = DateTime::<Local>::from(d);
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S");
    let to_write = format!("{} rasdf: {}\n", timestamp_str, message);

    if let Some(logfile) = &conf.logging {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logfile)?
            .write_all(to_write.as_bytes())?;
    } else if always {
        eprintln!("{}", to_write);
    }

    Ok(())
}
