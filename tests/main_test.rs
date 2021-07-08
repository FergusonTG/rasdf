use rasdf::*;

#[test]
fn add_two_rows() {
    let mut dbase = AsdfBase::new();
    let row_one = "/home/tim/tmp/|4.5|1234567|x";
    let row_two = "/home/tim/Documents/|6.7|1237890";
    let row_three = "";

    assert_eq!(1, dbase.add_line(row_one));
    assert_eq!(2, dbase.add_line(row_two));
    assert_eq!(2, dbase.add_line(row_three));
    assert!(! dbase.is_empty());
    assert_eq!(2, dbase.len());
}

#[test]
#[should_panic]
fn add_broken_row() {
    let mut dbase = AsdfBase::new();
    let row_one = "/home/tim/tmp/|4.5|1234567|x|";
    dbase.add_line(row_one);
    assert!(dbase.is_empty());
}


#[test]
fn find_row() {
    let mut dbase = AsdfBase::new();
    dbase.add_line("/home/tim/tmp/|4.5|1234567|x");
    dbase.add_line("/home/tim/Documents/|6.7|1237890");

    assert_eq!(vec!["/home/tim/tmp/"], dbase.find(&["tim", "tmp"]));
    assert_eq!(Vec::<&str>::new(), dbase.find(&["tim", "tmp", "reset"]));
}

#[test]
fn read_str_of_lines() {
    let lines = "/home/tim/tmp/|4.5|1234567|x\n\
                 /home/tim/Documents/|6.7|1237890|a\n\
                 /home/tim/mydiary|3.4|1236543|\n\
                 /root/src/|1.2|1235566|\n";
    let dbase = AsdfBase::from_data(lines);
    assert_eq!(4, dbase.len());
}

#[test]
fn read_data_file_successful() {
    let dbase = AsdfBase::from_file("./asdf.dat");
    assert_eq!(Vec::<&str>::new(), dbase.find(&["node", "nosuchfile.js"]));
}

#[test]
fn read_data_file_unsuccessful() {
    let dbase = AsdfBase::from_file("./asdf.dat");
    assert_eq!(Vec::<&str>::new(), dbase.find(&["node", "nosuchfile.js"]));
}

#[test]
fn add_new_data() {
    let dbase = &mut AsdfBase::from_file("./asdf.dat");
    let original_length = dbase.len();
    dbase.add("/tmp/newrow/newfile.doc", "");
    assert_eq!(original_length + 1, dbase.len());
}

#[test]
fn check_entry() {
    let dbase = &mut AsdfBase::from_file("./asdf.dat");
    let tdir = "/tmp/newrow/newfile.doc";

    dbase.add(tdir, "+");
    assert_eq!(1.0, dbase.entry(tdir).unwrap().rating);
    assert_eq!("+", dbase.entry(tdir).unwrap().flags);
}

#[test]
fn check_missing_entry() {
    let lines = "/home/tim/tmp/|4.5|1234567|x\n\
                 /home/tim/Documents/|6.7|1237890|a\n\
                 /home/tim/mydiary|3.4|1236543|\n\
                 /root/src/|1.2|1235566|\n";
    let dbase = AsdfBase::from_data(lines);
    assert!(dbase.entry("no/such/path").is_none());
}

#[test]
fn write_to_file() {
    let newfile = "./test_output.dat";
    let lines = "/home/tim/tmp/|4.5|1234567|x\n\
                 /home/tim/Documents/|6.7|1237890|a\n\
                 /home/tim/mydiary|3.4|1236543|\n\
                 /root/src/|1.2|1235566|\n";
    let dbase = AsdfBase::from_data(lines);

    assert!(dbase.write_out(newfile).is_ok());
    assert!(dbase.write_out("/no/such/file.dat").is_err());
}
