use rasdf::logging::{log, log_only};


fn main() {
    let conf = rasdf::config::Config::new();

    let usage = "
Commands: 
    help
    init
    clean
    add path [path...]
    remove path
    find segment [segment...]
    find-all segment [segment..]

Options:
    -a, -d, -f     find all resources, directories, files
    -D, -F, -R     use Date, Frecency, Rating
    -s, -l         strict or lax last-segment restriction
    -c, -i         case sensitive or insensitive
";

    // just to check what's going on...
    log_only(&conf, &format!("<{}>", rasdf::config::command_line()));

    match conf.command.as_str() {
        "init" => {
            let dbase = rasdf::AsdfBase::new();
            if let Err(e) = dbase.write_out(&conf) {
                log(&conf, &format!("Failed to write data file: {}", e));
            } else {
                log_only(&conf, "New database created.");
            }
        }

        "clean" => {
            let mut dbase = rasdf::AsdfBase::from_file(&conf);
            dbase.clean(&conf);
            if let Err(e) = dbase.write_out(&conf) {
                log(&conf, &format!("Failed to write data file: {}", e));
            } else {
                log_only(&conf,
                    &format!("Database cleaned; {} rows written.",
                        dbase.len())
                );
            }
        }

        "add" => {
            let mut dbase = rasdf::AsdfBase::from_file(&conf);

            // check that all arguments are not blacklisted
            for arg in &conf.arguments {
                for &cmd in &conf.cmd_blacklist {
                    if arg == cmd {
                        log_only(&conf,
                            &format!("Blacklisted command <{}>", arg));
                            return;
                    }
                }
            }
            for arg in &conf.arguments {
                dbase.add(&conf, &arg, "");
            }

            if let Err(e) = dbase.write_out(&conf) {
                log(&conf, &format!("Failed to write data file: {}", e));
            };  // don't log every addition!
        }

        "remove" => {
            let mut dbase = rasdf::AsdfBase::from_file(&conf);
            dbase.remove(&conf);
            if let Err(e) = dbase.write_out(&conf) {
                log(&conf,
                    &format!("Failed to write data file: {}", e));
            };
        }

        "find-all" => {
            let dbase = rasdf::AsdfBase::from_file(&conf);
            // eprintln!("Read {} lines.", dbase.len());

            let rets = dbase.find_list(&conf);
            for ret in rets.iter() {
                println!("{:6.4} {}", ret.1, ret.0);
            }
        }

        "find" => {
            let dbase = rasdf::AsdfBase::from_file(&conf);
            // eprintln!("Read {} lines.", dbase.len());

            if let Some(ret) = dbase.find(&conf) {
                println!("{}", ret);
            };
        }

        "help" => println!("{} COMMAND [OPTIONS...]\n{}", conf.executable, usage),

        &_ => {
            eprintln!("{}: not a valid command <{}>\n{}", 
                    conf.executable, conf.command, usage);
        }
    }
}
