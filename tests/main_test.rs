use std::path::PathBuf;
use std::time::SystemTime;

use rasdf::*;

fn make_config() -> config::Config<'static> {
    config::Config{
        version: "0.0.1.test".to_string(),
        executable: "test_harness".to_string(),
        command: String::new(),
        method: config::ScoreMethod::Frecency,
        datafile: PathBuf::from(".asdf.dat"),
        tempfile: PathBuf::from("/tmp"),
        maxlines: 20usize,
        logging: Some(PathBuf::from("./test-log.log")),
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        find_dirs: true,
        find_files: false,
        strict: true,
        case_sensitive: false,
        cmd_blacklist: Vec::new(),
        // entry_flags_add: Vec::new(),
        // entry_flags_remove: Vec::new(),
        arguments: Vec::new(),
    }
}

#[test]
fn test_fake_config() {
    let mut conf = make_config();
    conf.maxlines = 6;

    assert!(6 == conf.maxlines);
}

