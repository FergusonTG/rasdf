use std::env;


use rasdf;
// use crate::config::Config;
mod config;

fn main() {
    let mut argiter = env::args();
    let prog = argiter.next().unwrap_or(String::new());
    let cmnd = argiter.next().unwrap_or(String::new());
    let args: Vec<String> = argiter.collect();

    let config = config::Config::new();
    // eprintln!("Reading from {}", &filename);

    match cmnd.as_str() {
        "add" => {
            let mut dbase = rasdf::AsdfBase::from_file(&config);
            for path in args.iter() {
                dbase.add(path, "");
            };
            if let Err(e) = dbase.write_out(&config) {
                eprintln!("Failed to write data file: {}", e);
            };
        },
        "find-all" => {
            let dbase = rasdf::AsdfBase::from_file(&config);
            let rets = dbase.find_list(&config, &args);
            for ret in rets.iter() {
                println!("{:6.4} {}", ret.1, ret.0);
            };
        },
        "find" => {
            let dbase = rasdf::AsdfBase::from_file(&config);
            if let Some(ret) = dbase.find(&config, &args) {
                println!("{}", ret);
            };
        },

        &_ => eprintln!("{}: not a valid command <{}>", prog, cmnd),
    }
}
