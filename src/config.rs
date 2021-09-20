// Setup config object to read env variables
// and some local defaults.

use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Copy, Clone, Debug)]
pub enum ScoreMethod { Date, Rating, Frecency }

impl ScoreMethod {
    fn from(s: &str) -> Self {
        match s {
                "date"     => ScoreMethod::Date,
                "rating"   => ScoreMethod::Rating,
                "frecency" => ScoreMethod::Frecency,
                _          => ScoreMethod::Frecency,
        }
    }
}

pub struct Config<'a> {
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
    pub cmd_blacklist: Vec<&'a str>,
    pub arguments: Vec<String>,
}

impl Config<'_>  {
    pub fn new() -> Config<'static> {
        let mut config = Config {
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
            maxlines: {
                let mut maxlines = 200usize;
                if let Ok(stringval) = env::var("RASDF_MAXLINES") {
                    if let Ok(intval) = stringval.parse::<usize>() {
                        maxlines = intval;
                }};
                maxlines
            },
            logging: match env::var("RASDF_LOGFILE") {
                Ok(filepath)   => Some(PathBuf::from(filepath)),
                _              => None,
            },
            current_time: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
            find_dirs: true,
            find_files: false,
            strict: true,
            case_sensitive: true,
            cmd_blacklist: ["rasdf", "ls", "dir", "vdir", "ddir", "cd", "rm", "rmdir", "tree"].to_vec(),
            arguments: vec![],
        };

        // override from $RASDF_FLAGS
        if let Ok(flags) = env::var("RASDF_FLAGS") {
            for flag in flags.chars() {
                config.set_flag(flag);
            }
        }

        // overriding by command line flags...
        let mut argiter = env::args().peekable();

        // first arguments are the executable and the command
        config.executable = argiter.next().unwrap();
        config.command = argiter.next().unwrap();

        while let Some(arg) = argiter.peek() {
            if ! arg.starts_with('-') { break; }
            for flag in arg.chars().skip(1) {
                config.set_flag(flag);
            }
            
            argiter.next();
        }

        // remainder are collected into arguments
        config.arguments = argiter.collect();

        // return the config
        config
    }

    fn set_flag(&mut self, flag: char) {
            match flag {
                'a' => {self.find_dirs = true; self.find_files = true;},
                'd' => {self.find_dirs = true; self.find_files = false;},
                'f' => {self.find_dirs = false; self.find_files = true;},
                'D' => self.method = ScoreMethod::Date,
                'F' => self.method = ScoreMethod::Frecency,
                'R' => self.method = ScoreMethod::Rating,
                's' => self.strict = true,
                'l' => self.strict = false,
                'c' => self.case_sensitive = true,
                'i' => self.case_sensitive = false,

                _   => panic!("Unrecognised option {}", flag),
        };
    }
}
    


