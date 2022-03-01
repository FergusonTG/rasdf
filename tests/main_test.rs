use std::path::PathBuf;
use std::time::SystemTime;

use rasdf::*;

#[test]
fn add_two_rows() {
    let conf = make_config();
    let mut dbase = AsdfBase::new();
    let row_one = "/home/tim/tmp|4.5|1234567|x";
    let row_two = "/home/tim/Documents|6.7|1237890";
    let row_three = "";

    assert_eq!(1, dbase.add_line(&conf, row_one));
    assert_eq!(2, dbase.add_line(&conf, row_two));
    assert_eq!(2, dbase.add_line(&conf, row_three));
    assert!(!dbase.is_empty());
    assert_eq!(2, dbase.len());
}

#[test]
fn add_broken_row() {
    let conf = make_config();
    let mut dbase = AsdfBase::new();
    let row_one = "/home/tim/tmp|4.5|1234567|x|";
    dbase.add_line(&conf, row_one);
    assert!(dbase.is_empty());
}

#[test]
fn read_str_of_lines() {
    let conf = make_config();
    let lines = "/home/tim/tmp/|4.5|1234567|x\n\
                 /home/tim/Documents/|6.7|1237890|a\n\
                 /home/tim/Downloads|3.4|1236543|\n\
                 /usr/bin/|1.2|1235566|\n";
    let dbase = AsdfBase::from_data(&conf, lines);
    assert_eq!(4, dbase.len());
}

#[test]
fn read_data_file_successful() {
    let mut conf = make_config();
    conf.datafile = PathBuf::from("./asdf.dat");

    let dbase = AsdfBase::from_file(&conf);
    assert!(!dbase.is_empty());
}

#[test]
fn read_data_file_unsuccessful() {
    let mut conf = make_config();
    conf.datafile = PathBuf::from("./missing.file");

    let dbase = AsdfBase::from_file(&conf);
    assert!(dbase.is_empty());
}

#[test]
fn find_rows() {
    let conf = make_config();
    let mut dbase = AsdfBase::new();
    dbase.add_line(&conf, "/home/tim/tmp|4.5|1234567|x");
    dbase.add_line(&conf, "/home/tim/Documents|6.7|1237890");
    dbase.add_line(&conf, "/home/tim/Documents/2019|6.7|1237890");

    let mut conf = make_config();
    conf.arguments = vec![String::from("tim"), String::from("m")];
    assert_eq!(2, dbase.find_list(&conf).len());

    conf.method = config::ScoreMethod::Date;
    conf.arguments = vec![String::from("tmp")];
    assert_eq!(vec![("/home/tim/tmp", 1234567_f32)], dbase.find_list(&conf));
}

#[test]
fn find_row() {
    let mut conf = make_config();
    let mut dbase = AsdfBase::new();
    dbase.add_line(&conf, "/home/tim/tmp|9.5|1234567|x");
    dbase.add_line(&conf, "/home/tim/Documents|9.7|1237890");
    dbase.add_line(&conf, "/home/tim/Private/tmp|4.7|1236654");

    conf.arguments = vec![String::from("tim"), String::from("tmp")];
    // refactor AsdfBase::find to return an Option<&str>
    assert_eq!(Some("/home/tim/tmp"), dbase.find(&conf));

    conf.arguments.push(String::from("missing"));
    assert!(dbase.find(&conf).is_none());
}

#[test]
fn read_dirs_only() {
    let mut conf = make_config();
    let lines = "/home/tim/tmp|4.5|1234567|x\n\
                 /home/tim/tmp/one.file|6.5|1237890|f\n\
                 /home/tim/tmp/two.file|5.5|1237890|a\n\
                 /home/tim/tmp/three.file|5.5|1237890|a";
    let dbase = AsdfBase::from_data(&conf, lines);
    assert_eq!(4, dbase.len());

    conf.find_dirs = true;
    conf.find_files = false;
    conf.strict = false;
    conf.arguments = vec![String::from("tim"), String::from("tmp")];

    let result = dbase.find_list(&conf);
    assert_eq!(1, result.len());
    assert_eq!("/home/tim/tmp", result[0].0);
}

#[test]
fn read_files_only() {
    let mut conf = make_config();
    let lines = "/home/tim/tmp|4.5|1234567|x\n\
                 /home/tim/tmp/one.file|6.5|1237890|f\n\
                 /home/tim/tmp/two.file|5.5|1237890|a\n\
                 /home/tim/tmp/three.file|5.5|1237890|a";
    let dbase = AsdfBase::from_data(&conf, lines);
    assert_eq!(4, dbase.len());

    conf.find_dirs = false;
    conf.find_files = true;
    conf.strict = false;
    conf.arguments = vec![String::from("tmp"), String::from("fil")];
    let result = dbase.find_list(&conf);
    assert_eq!(3, result.len());
}

#[test]
fn read_case_sensitive() {
    let mut conf = make_config();
    let lines = "/home/tim/tmp|4.5|1234567|x\n\
                 /home/tim/tmp/one.file|6.5|1237890|f\n\
                 /home/tim/tmp/two.file|5.5|1237890|a\n\
                 /home/tim/tmp/three.file|5.5|1237890|a";
    let dbase = AsdfBase::from_data(&conf, lines);
    assert_eq!(4, dbase.len());

    conf.find_dirs = false;
    conf.find_files = true;
    conf.strict = false;
    conf.case_sensitive = true;

    conf.arguments = vec![String::from("tmp"), String::from("fil")];
    let result = dbase.find_list(&conf);
    assert_eq!(3, result.len());

    conf.arguments = vec![String::from("TMP"), String::from("fil")];
    let result = dbase.find_list(&conf);
    assert_eq!(0, result.len());
}

#[test]
fn add_new_data() {
    let mut conf = make_config();
    conf.datafile = PathBuf::from("./asdf.dat");
    conf.arguments.push(
        PathBuf::from("./")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    );

    let mut dbase = AsdfBase::from_file(&conf);
    dbase.remove(&conf);

    let original_length = dbase.len();
    dbase.add(&conf, "./", "");
    assert_ne!(original_length, dbase.len());
}

#[test]
fn check_entry() {
    let mut dbase = AsdfBase::new();
    let conf = make_config();

    let tdir = "/home/tim/tmp";
    dbase.add(&conf, tdir, "+");

    assert!(dbase.entry(tdir).is_some());

    assert_eq!(1.0, dbase.entry(tdir).unwrap().rating);
    assert_eq!("+", dbase.entry(tdir).unwrap().flags);

    // check that add is not overriding existing flag
    dbase.add(&conf, tdir, "");
    assert_eq!("+", dbase.entry(tdir).unwrap().flags);
}

#[test]
fn check_scoring() {
    let conf = make_config();

    let dbase = AsdfBase::from_data(
        &conf,
        format!("/home/tim/tmp|1.5|{}|x\n", conf.current_time).as_str(),
    );

    assert_eq!(9.0, dbase.entry("/home/tim/tmp").unwrap().score(&conf));
}

#[test]
fn check_missing_entry() {
    let conf = make_config();
    let lines = "/home/tim/tmp/|4.5|1234567|x\n\
                 /home/tim/Documents/|6.7|1237890|a\n\
                 /home/tim/mydiary|3.4|1236543|\n\
                 /root/src/|1.2|1235566|\n";
    let dbase = AsdfBase::from_data(&conf, lines);
    assert!(dbase.entry("no/such/path").is_none());
}

#[test]
fn clean_dbase() {
    let mut conf = make_config();
    conf.maxlines = 5;

    let lines = "/home/tim/tmp/|4.5|1234567|x\n\
                 /home/tim/tmp/one.file|1|1234568|\n\
                 /home/tim/tmp/two.file|1|1234568|\n\
                 /home/tim/tmp/three.file|1|1234568|\n\
                 /home/tim/Documents/|6.7|1237890|a\n\
                 /home/tim/mydiary|3.4|1236543|\n\
                 /usr/bin/|1.2|1235566|\n";
    let mut dbase = AsdfBase::from_data(&conf, lines);

    assert!(dbase.clean(&conf));
    assert!(dbase.len() <= conf.maxlines);
}

#[test]
fn write_to_file() {
    let mut conf = make_config();
    conf.datafile = PathBuf::from("./test_output.dat");

    let lines = "/home/tim/tmp/|4.5|1234567|x\n\
                 /home/tim/Documents/|6.7|1237890|a\n\
                 /home/tim/mydiary|3.4|1236543|\n\
                 /root/src/|1.2|1235566|\n";
    let dbase = AsdfBase::from_data(&conf, lines);

    assert!(dbase.write_out(&conf).is_ok());

    conf.datafile = PathBuf::from("/no/such/file.dat");
    assert!(dbase.write_out(&conf).is_err());
}

fn make_config() -> config::Config<'static> {
    config::Config {
        executable: "test_harness".to_string(),
        command: String::new(),
        method: config::ScoreMethod::Frecency,
        datafile: PathBuf::from(".asdf.dat"),
        tempfile: PathBuf::from("/tmp"),
        maxlines: 200usize,
        logging: None,
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        find_dirs: true,
        find_files: false,
        strict: true,
        case_sensitive: false,
        cmd_blacklist: Vec::new(),
        arguments: Vec::new(),
    }
}
