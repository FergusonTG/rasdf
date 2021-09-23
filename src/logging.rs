
use crate::config::Config;

use std::fs;
use std::io::Write;

extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Local;
use std::time::{UNIX_EPOCH, Duration};

/// if there's a log file, write to it.

pub fn log(conf: &Config, message: &str) {
    write_log(conf, message).expect("Failed writing to log file");
}

fn write_log(conf: &Config, message: &str)  -> std::io::Result<()> {
    // Creates a new SystemTime from the specified number of whole seconds
    let d = UNIX_EPOCH + Duration::from_secs(conf.current_time);
    // Create DateTime from SystemTime
    let datetime = DateTime::<Local>::from(d);
    // Formats the combined date and time with the specified format string.
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S"); // .to_string();
    // Combine into a new string
    let to_write = format!("{} rasdf: {}\n", timestamp_str, message);

    if let Some(logfile) = &conf.logging {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logfile)?
            .write_all(to_write.as_bytes())?;
    } else {
        eprintln!("{}", to_write);
    }

    Ok(())

}


