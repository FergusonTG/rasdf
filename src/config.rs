// Setup config object to read env variables
// and some local defaults.

use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

const VERSION: &str = "0.1.2";

#[derive(Copy, Clone, Debug)]
pub enum ScoreMethod {
    Date,
    Rating,
    Frecency,
}

impl ScoreMethod {
    fn from(s: &str) -> Self {
        match s {
            "date" => ScoreMethod::Date,
            "rating" => ScoreMethod::Rating,
            "frecency" => ScoreMethod::Frecency,
            _ => ScoreMethod::Frecency,
        }
    }
}

pub struct Config<'a> {
    pub version: String,
    pub executable: String,
    pub command: String,
    pub method: ScoreMethod,
    pub datafile: PathBuf,
    pub tempfile: PathBuf,
    pub maxlines: usize,
    pub logging: Option<PathBuf>,
    pub current_time: u64,
    pub find_dirs: bool,
    pub find_files: bool,
    pub strict: bool,
    pub case_sensitive: bool,
    pub flags: String,
    pub cmd_blacklist: Vec<&'a str>,
    pub arguments: Vec<String>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Config<'_> {
    pub fn new() -> Config<'static> {
        let mut config = Config {
            version: String::from(VERSION),
            executable: String::new(),
            command: String::new(),
            method: if let Ok(s) = env::var("RASDF_METHOD") {
                ScoreMethod::from(&s)
            } else {
                ScoreMethod::Frecency
            },
            datafile: if let Ok(filepath) = env::var("RASDF_DATAFILE") {
                PathBuf::from(filepath)
            } else {
                PathBuf::from("/home/tim/.config/rasdf/rasdf.dat")
            },
            tempfile: PathBuf::from("/tmp/rasdf_tf.dat"),
            maxlines: match env::var("RASDF_MAXLINES").map(|var| var.parse::<usize>()) {
                Ok(Ok(maxlines)) => maxlines,
                _ => 200,
            },
            logging: match env::var("RASDF_LOGFILE") {
                Ok(filepath) => Some(PathBuf::from(filepath)),
                _ => None,
            },
            current_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap() // okay because current time !< epoch
                .as_secs(),
            find_dirs: true,
            find_files: false,
            strict: true,
            case_sensitive: true,
            flags: String::new(),
            cmd_blacklist: [
                "rasdf", "ls", "dir", "vdir", "ddir", "cd", "rm", "rmdir", "tree",
            ].to_vec(),
            arguments: vec![],
        };

        // override from $RASDF_FLAGS
        if let Ok(cli_flags) = env::var("RASDF_FLAGS") {
            for cli_flag in cli_flags.chars() {
                config.set_cli_flag(cli_flag);
            }
        }

        // overriding by command line flags...
        let mut argiter = env::args().peekable();

        // first arguments are the executable and the command
        config.executable = argiter.next().unwrap_or_default();
        config.command = argiter.next().unwrap_or_default();

        while let Some(arg) = argiter.peek() {
            if !arg.starts_with('-') {
                break;
            }
            for cli_flag in arg.chars().skip(1) {
                config.set_cli_flag(cli_flag);
            }

            argiter.next();
        }

        // remainder are collected into arguments
        config.arguments = argiter.collect();

        // return the config
        config
    }

    fn set_cli_flag(&mut self, cli_flag: char) {
        match cli_flag {
            'a' => {
                self.find_dirs = true;
                self.find_files = true;
            }
            'd' => {
                self.find_dirs = true;
                self.find_files = false;
            }
            'f' => {
                self.find_dirs = false;
                self.find_files = true;
            }
            'D' => self.method = ScoreMethod::Date,
            'F' => self.method = ScoreMethod::Frecency,
            'R' => self.method = ScoreMethod::Rating,
            's' => self.strict = true,
            'l' => self.strict = false,
            'c' => self.case_sensitive = true,
            'i' => self.case_sensitive = false,

            _ => panic!("Unrecognised option {}", cli_flag),
        };
    }
}

/// Return the whole command line as seen by env::args
pub fn command_line() -> String {
    env::args().fold(String::new(), |mut s, arg| {
        s.push_str(&arg);
        s.push(' ');
        s
    })
}

/// Return the user's home directory
pub fn home_dir() -> Option<String> {
    env::var("HOME").ok()
}
