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

/// RasdfBaseData
///
/// Data for a single path
/// fields:
/// + rating: float, continually updated when cleaning file
/// + date: u64, a UNIX-style datestamp of last access
/// + flags: string, use to be determined
///
#[derive(Debug)]
pub struct RasdfBaseData {
    pub rating: f32,
    pub date: u64,
    pub flags: String,
}

impl RasdfBaseData {
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

/// RasdfBase
///
/// Database of all the mappings
/// path -> RasdfBaseData
///
/// path is maintained as absolute canonical String
///
pub struct RasdfBase {
    contents: HashMap<String, RasdfBaseData>,
}

impl RasdfBase {
    pub fn new() -> RasdfBase {
        RasdfBase {
            contents: HashMap::new(),
        }
    }
}

impl Default for RasdfBase {
    fn default() -> RasdfBase {
        RasdfBase::new()
    }
}

impl RasdfBase {
    /// number of records in database
    pub fn len(&self) -> usize {
        self.contents.len()
    }

    /// true if database has no records
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    /// return basedata for given path, or None
    pub fn entry(&self, path: &str) -> Option<&RasdfBaseData> {
        self.contents.get(path)
    }

    /// add or update a path record in the database
    pub fn add(&mut self, conf: &Config, path: &str) {
        let pathstring = if let Some(checkpath) = canonical_string(path) // Option<PathBuf>
            .map(|pb| pb.into_os_string()) // Option<OsString>
            .and_then(|s| s.into_string().ok())
        {
            // Option<String>
            checkpath
        } else {
            return;
        };

        if let Some(data) = self.contents.get_mut(&pathstring) {
            log_only(conf, &format!("Uprating path: {}", pathstring));
            data.rating += 1.0 / data.rating;
            data.date = conf.current_time;
            if ! &conf.entry_flags_add.is_empty() || ! conf.entry_flags_remove.is_empty() {
                let mut flags: Vec<char> = data.flags.chars().collect();
                vec_insert(&mut flags, &conf.entry_flags_add, &conf.entry_flags_remove);
                data.flags = flags.iter().collect();
            }
        } else {
            log_only(conf, &format!("Adding new path: {}", pathstring));
            let mut flags = Vec::<char>::new();
            vec_insert(&mut flags, &conf.entry_flags_add, &conf.entry_flags_remove);
            self.contents.insert(
                pathstring,
                RasdfBaseData {
                    rating: 1.0,
                    date: conf.current_time,
                    flags: flags.iter().collect(),
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
            let pathstring = if let Some(checkpath) =
                canonical_string(v[0]) // Option<PathBuf>
                    .map(|pb| pb.into_os_string()) // Option<OsString>
                    .and_then(|s| s.into_string().ok())
            {
                // Option<String>
                checkpath
            } else {
                return self.contents.len();
            };

            if let (Ok(rating), Ok(date), flags) =
                (v[1].parse::<f32>(), v[2].parse::<u64>(), v[3].to_string())
            {
                self.contents.insert(
                    pathstring,
                    RasdfBaseData {
                        rating,
                        date,
                        flags,
                    },
                );
            };
        } else {
            log(conf, &format!("Can't parse row: {}", row));
        }

        self.contents.len()
    }

    pub fn from_data(conf: &Config, lines: &str) -> RasdfBase {
        let mut dbase = RasdfBase::new();
        for line in lines.split('\n') {
            dbase.add_line(conf, line);
        }
        dbase
    }

    pub fn from_file(conf: &Config) -> RasdfBase {
        if let Ok(contents) = fs::read_to_string(&conf.datafile) {
            // eprintln!("{}", &contents);
            RasdfBase::from_data(conf, &contents)
        } else {
            RasdfBase::new()
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
        true
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
        fs::rename(&path, &conf.datafile).or_else(|_| fs::copy(&path, &conf.datafile).map(|_| ()))
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
            let mut ord = a.1.partial_cmp(&b.1).unwrap();
            if ord == Ordering::Equal {
                ord = a.0.cmp(b.0);
            }
            ord
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

/// For vector v, add elements in a and remove elements in r
/// Note that elements are added before they are removed.
fn vec_insert<T: std::cmp::PartialEq + Copy>(v: &mut Vec<T>, a: &[T], r: &[T]) {
    a.iter().for_each(|c| {
        if !v.contains(c) {
            v.push(*c);
        }
    });
    v.retain(|c| !r.contains(c));
}


fn canonical_string(path: &str) -> Option<PathBuf> {
    // return a Some(String) if path is a real path
    // otherwise None
    let mut pathstring = String::from(path);
    if pathstring.starts_with('~') {
        pathstring = pathstring.replacen('~', &home_dir()?, 1)
    }

    fs::canonicalize(&pathstring).ok()
}
