// Setup config object to read env variables
// and some local defaults.

use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Copy, Clone, Debug)]
enum ScoreMethod { Date, Rating, Frecency }

impl ScoreMethod {
    pub fn from(s: &str) -> Self {
        match s {
                "date"     => ScoreMethod::Date,
                "rating"   => ScoreMethod::Rating,
                "frecency" => ScoreMethod::Frecency,
                _          => ScoreMethod::Frecency,
        }
    }
}

pub struct Config {
    pub method: ScoreMethod,
    pub datafile: PathBuf,
    pub tempfile: PathBuf,
    pub logging: Option<PathBuf>,
    pub current_time: u64,
    pub cmd_blacklist: Vec<String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            method: if let Ok(s) = env::var("RASDF_METHOD") {
                ScoreMethod::new(s)
            } else {
                ScoreMethod::Frecency
            },
            datafile: match env::var("RASDF_DATAFILE") {
                Ok(filepath)   => PathBuf::from(filepath),
                _              => PathBuf::from("$HOME/.config/rasdf/rasdf.dat"),
            },
            tempfile: PathBuf::from("/tmp/rasdf_tf.dat"),
            logging: match env::var("RASDF_LOGFILE") {
                Ok(filepath)   => Some(PathBuf::from(filepath)),
                _              => None,
            },
            current_time: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
            cmd_blacklist: ["ls", "dir", "vdir", "ddir", "cd", "rm", "rmdir", "tree"].map(|s| String::from(s)).collect(),
        }
    }
}
    


