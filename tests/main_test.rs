use std::path::PathBuf;

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
        current_time: 123456,
        find_dirs: true,
        find_files: false,
        strict: true,
        case_sensitive: false,
        flags: String::new(),
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

#[test]
fn test_make_database() {
    let conf = make_config();

    let mut dbase = AsdfBase::new();
    dbase.add_line(&conf, "/home/tim/tmp|2.2|123456|t");
    assert!( dbase.len() == 1 );
}

#[test]
fn test_basedata_new() {
    let conf = make_config();

    let _ = AsdfBaseData::new( &conf, Some(1.5), Some(123456), "tf",);
    let _ = AsdfBaseData::new( &conf, None, None, "" );
}

#[test]
fn test_basedata_update() {
    let conf = make_config();

    let mut bdata = AsdfBaseData::new(
        &conf, Some(4.0), Some(121212), "tf"
    );
    bdata.update_with(&AsdfBaseData::new(&conf, None, None, "tg"));

    assert_eq!(bdata.flags, "tfg");
    assert_eq!(bdata.date, conf.current_time);
    assert_eq!(bdata.rating, 4.25);
}

#[test]
fn test_add_from_string() {
    let conf = make_config();

    let mut dbase = AsdfBase::new();
    dbase.add_line(&conf, "/home/tim/tmp|2.2|123456|t");
    assert!( dbase.len() == 1 );
}

#[test]
fn test_new_from_string() {
    let conf = make_config();

    let mut dbase = AsdfBase::from_data(&conf, 
        "/home/tim/tmp|2.2|123456|td\n/home/tim/|1.9|123457|td"
    );
    assert!( dbase.len() == 2 );
    
    dbase.add_line(&conf, "/home/tim/tmp|2.2|123456|t");
    assert!( dbase.len() == 2 );
}
