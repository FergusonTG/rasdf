use std::env;
use std::fs;
use std::io::Write;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{PathBuf, MAIN_SEPARATOR};

pub mod config;
use config::{home_dir, Config, ScoreMethod};

pub mod logging;
use logging::{log, log_only};

/// AsdfBaseData
/// 
/// Data for a single path
/// fields:
/// + rating: float, continually updated when cleaning file
/// + date: u64, a UNIX-style datestamp of last access
/// + flags: string, use to be determined
/// 
#[derive(Debug)]
pub struct AsdfBaseData {
    pub rating: f32,
    pub date: u64,
    pub flags: String,
}

impl AsdfBaseData {
    pub fn score(&self, conf: &Config) -> f32 {
        match conf.method {
            ScoreMethod::Date => self.date as f32,
            ScoreMethod::Rating => self.rating,
            ScoreMethod::Frecency => {
                self.rating
                    * match conf.current_time - self.date {
                        0..=3600 => 6.0,
                        3601..=86400 => 4.0,
                        86401..=604800 => 2.0,
                        _ => 1.0,
                    }
            }
        }
    }
}

/// AsdfBase
/// 
/// Database of all the mappings
/// path -> AsdfBaseData
/// 
/// path is maintained as absolute canonical String
///
pub struct AsdfBase {
    contents: HashMap<String, AsdfBaseData>,
}

impl AsdfBase {
    pub fn new() -> AsdfBase {
        AsdfBase {
            contents: HashMap::new(),
        }
    }
}

impl Default for AsdfBase {
    fn default() -> AsdfBase {
        AsdfBase::new()
    }
}

impl AsdfBase {
    /// number of records in database
    pub fn len(&self) -> usize {
        self.contents.len()
    }

    /// true if database has no records
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    /// return basedata for given path, or None
    pub fn entry(&self, path: &str) -> Option<&AsdfBaseData> {
        self.contents.get(path)
    }

    /// add or update a path record in the database
    pub fn add(&mut self, conf: &Config, path: &str, flags: &str) {
        let pathstring = match canonical_string(path) {
            Some(pathbuf) => match pathbuf.to_str() {
                Some(pathstr) => pathstr.to_string(),
                None => return,
            },
            None => return,
        };

        if let Some(data) = self.contents.get_mut(&pathstring) {
            log_only(conf, &format!("Uprating path: {}", pathstring));
            data.rating += 1.0 / data.rating;
            data.date = conf.current_time;
            for c in flags.chars() {
                if !data.flags.contains(c) {
                    data.flags.push(c);
                }
            }
        } else {
            log_only(conf, &format!("Adding new path: {}", pathstring));
            self.contents.insert(
                pathstring,
                AsdfBaseData {
                    rating: 1.0,
                    date: conf.current_time,
                    flags: String::from(flags),
                },
            );
        }
    }

    pub fn remove(&mut self, conf: &Config) {
        if self.contents.remove(&conf.arguments[0]).is_none() {
            log(
                conf,
                &format!("Could not find row to remove: {}", &conf.arguments[0]),
            );
        }
    }

    pub fn add_line(&mut self, conf: &Config, row: &str) -> usize {
        // ignore a blank line
        if row.is_empty() {
            return self.contents.len();
        }

        let mut v: Vec<&str> = row.split('|').collect();
        if v.len() == 3 {
            v.push("");
        }
        if v.len() == 4 {
            let pathstring = match canonical_string(v[0]) {
                Some(pathbuf) => match pathbuf.to_str() {
                    Some(pathstr) => pathstr.to_string(),
                    None => return self.contents.len(),
                },
                None => return self.contents.len(),
            };

            if let (Ok(rating), Ok(date), flags) =
                (v[1].parse::<f32>(), v[2].parse::<u64>(), v[3].to_string())
            {
                self.contents.insert(
                    pathstring,
                    AsdfBaseData {
                        rating,
                        date,
                        flags,
                    },
                );
            };
        } else {
            log(&conf, &format!("Can't parse row: {}", row));
        }

        self.contents.len()
    }

    pub fn from_data(conf: &Config, lines: &str) -> AsdfBase {
        let mut dbase = AsdfBase::new();
        for line in lines.split('\n') {
            dbase.add_line(conf, line);
        }
        dbase
    }

    pub fn from_file(conf: &Config) -> AsdfBase {
        if let Ok(contents) = fs::read_to_string(&conf.datafile) {
            // eprintln!("{}", &contents);
            AsdfBase::from_data(conf, &contents)
        } else {
            AsdfBase::new()
        }
    }

    pub fn clean(&mut self, conf: &Config) -> bool {
        if self.len() <= conf.maxlines {
            log_only(conf, "Nothing to clean");
            return false;
        };

        // Adjust all ratings down by 10%
        for rec in self.contents.values_mut() {
            rec.rating *= 0.9;
        }

        // A list of (path, rating) tuples sorted on rating
        let mut keys: Vec<_> = self
            .contents
            .keys()
            .map(|f| (f.to_string(), self.contents[f].rating))
            .collect();
        keys.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // just keep the ones beyond MAXLINES
        // and remove them from the database.
        keys.truncate(keys.len() - conf.maxlines);
        let keys_truncated = keys.len();
        for f in keys.iter() {
            self.contents.remove(&f.0);
        }
        log_only(conf, &format!("{} records truncated", keys_truncated));
        return true;
    }

    pub fn write_out(&self, conf: &Config) -> std::io::Result<()> {
        let path = env::temp_dir().join("myfile.txt");
        // println!("Writing to temp file {:?}", path);
        let mut buffer = fs::File::create(&path)?;

        // write data out to temp file
        for (key, value) in &self.contents {
            buffer.write_all(
                format!("{}|{}|{}|{}\n", key, value.rating, value.date, value.flags).as_bytes(),
            )?;
        }

        // and copy that back to proper place
        // println!("Copying to {:?}", &conf.datafile);
        fs::rename(&path, &conf.datafile).or(match fs::copy(&path, &conf.datafile) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        })
    }

    pub fn find_list(&self, conf: &Config) -> Vec<(&str, f32)> {
        let mut v = Vec::<&str>::new();

        let elements: Vec<String> = match conf.case_sensitive {
            true => conf.arguments.clone(),
            false => conf.arguments.iter().map(|s| s.to_lowercase()).collect(),
        };

        'paths: for path in self.contents.keys() {
            // make a mutable copy to turn into lowercase.
            let mut pathstring = path.to_string();

            // check if we're looking for dirs or folders (or both)
            let p = PathBuf::from(&path);
            if (!conf.find_dirs && p.is_dir()) || (!conf.find_files && p.is_file()) {
                continue 'paths;
            }

            // if case insensitive, convert everything to lowercase
            if !conf.case_sensitive {
                pathstring = pathstring.to_lowercase();
            };

            let mut start = 0usize;

            for element in elements.iter() {
                if let Some(p) = pathstring[start..].find(element) {
                    start += p + element.len();
                } else {
                    continue 'paths;
                }
            }
            // only add the path if the last element was in the last segment.
            if !conf.strict || pathstring[start..].find(MAIN_SEPARATOR).is_none() {
                v.push(path);
            }
        }
        // collect each path into a (path , score) tuple
        // Using unwrap is okay because the path is definitely in the database.
        let mut result: Vec<(_, _)> = v
            .iter()
            .map(|path| (*path, self.entry(path).unwrap().score(conf)))
            .collect();

        // Sort the results according to the score first then path
        result.sort_by(|a, b| {
            let ord = a.1.partial_cmp(&b.1).unwrap();
            match ord {
                Ordering::Equal => a.0.cmp(b.0),
                _ => ord,
            }
        });
        result
    }

    pub fn find(&self, conf: &Config) -> Option<&str> {
        let v = self.find_list(conf);

        // go through list of tuples and track the highest scoring
        let mut ret: (Option<&str>, f32) = (None, f32::NAN);
        for t in v {
            if ret.0.is_none() || ret.1.lt(&t.1) {
                ret = (Some(t.0), t.1);
            }
        }
        ret.0
    }
}

fn canonical_string(path: &str) -> Option<PathBuf> {
    // return a String if path is a real path
    // otherwise None
    let pathstring = if path.starts_with('~') {
        if let Some(homedir) = home_dir() {
            path.replacen('~', &homedir, 1)
        } else {
            return None;
        }
    } else {
        String::from(path)
    };

    fs::canonicalize(&pathstring).ok()
}
