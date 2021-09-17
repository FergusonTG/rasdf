
use rasdf;

fn main() {
    let conf = rasdf::config::Config::new();

    match conf.command.as_str() {
        "init" => {
            let dbase = rasdf::AsdfBase::new();
            if let Err(e) = dbase.write_out(&conf) {
                eprintln!("Failed to write data file: {}", e);
            };
        },

        "clean" => {
            let mut dbase = rasdf::AsdfBase::from_file(&conf);
            dbase.clean(&conf);
            if let Err(e) = dbase.write_out(&conf) {
                eprintln!("Failed to write data file: {}", e);
            };
        },

        "add" => {

            let mut dbase = rasdf::AsdfBase::from_file(&conf);

            for arg in conf.arguments.iter() {
                if conf.cmd_blacklist.iter().any(|s| arg == s) { return; };
                dbase.add(&conf, arg, "");
            };
            if let Err(e) = dbase.write_out(&conf) {
                eprintln!("Failed to write data file: {}", e);
            };
        },

        "remove" => {
            let mut dbase = rasdf::AsdfBase::from_file(&conf);
            dbase.remove(&conf.arguments[0]);
            if let Err(e) = dbase.write_out(&conf) {
                eprintln!("Failed to write data file: {}", e);
            };
        }

        "find-all" => {
            let dbase = rasdf::AsdfBase::from_file(&conf);
            // eprintln!("Read {} lines.", dbase.len());

            let rets = dbase.find_list(&conf, &conf.arguments);
            for ret in rets.iter() {
                println!("{:6.4} {}", ret.1, ret.0);
            };
        },

        "find" => {
            let dbase = rasdf::AsdfBase::from_file(&conf);
            // eprintln!("Read {} lines.", dbase.len());

            if let Some(ret) = dbase.find(&conf, &conf.arguments) {
                println!("{}", ret);
            };
        },

        &_ => eprintln!("{}: not a valid command <{}>", conf.executable, conf.command),
    }
}
