use std::env;


use rasdf;

fn main() {
    let mut argiter = env::args();
    let prog = argiter.next().unwrap_or(String::new());
    let cmnd = argiter.next().unwrap_or(String::new());
    let args: Vec<String> = argiter.collect();

    let method = rasdf::ScoreMethod::Frecency;

    let filename = rasdf::default_datafile();
    //let filename = "./asdf.dat";
    println!("Reading from {}\n", &filename);

    match cmnd.as_str() {
        "add" => {
            let mut dbase = rasdf::AsdfBase::from_file(&filename);
            for path in args.iter() {
                dbase.add(path, "");
            };
            let _ = dbase.write_out(&filename);
        },
        "find" => {
            let dbase = rasdf::AsdfBase::from_file(&filename);
            let rets = dbase.find_list(method, &args);
            for ret in rets.iter() {
                println!("{:6.4} {}", ret.1, ret.0);
            };
        }
        &_ => println!("{}: not a valid command <{}>\n", prog, cmnd),
    }
}
