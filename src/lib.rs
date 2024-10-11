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

// TODO: Replace RasdfBase with RasdfBase throughout code base...

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
    pub fn new(conf: &Config,
        opt_rating: Option<f32>,
        opt_date: Option<u64>,
        flags: &str ) -> RasdfBaseData {
        RasdfBaseData {
            rating: opt_rating.unwrap_or(1.0),
            date: opt_date.unwrap_or(conf.current_time),
            flags: flags.to_string(),
        }
    }

    pub fn update_with(&mut self, other: &RasdfBaseData) {
        self.rating += other.rating / self.rating;
        self.date = std::cmp::max(self.date, other.date);
        self.flags = {
            let mut set: Vec<char> = Vec::new();
            for ch in self.flags.chars() {
                if !set.contains(&ch) { set.push(ch) };
            }
            for ch in other.flags.chars() {
                if !set.contains(&ch) { set.push(ch) };
            }
            set.iter().collect()
        };
    }

    pub fn score(&self, conf: &Config) -> f32 {
        match conf.method {
            ScoreMethod::Date => self.date as f32,
            ScoreMethod::Rating => self.rating,
            ScoreMethod::Frecency => {
                let scale: f32 = match conf.current_time - self.date {
                        0..=3600 => 6.0,        // less than an hour
                        3601..=86400 => 4.0,    // up to one day
                        86401..=604800 => 2.0,  // up to seven days
                        _ => 1.0,               // otherwise...
                    };
                scale * self.rating
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
    pub fn add_path(&mut self, conf: &Config, path: &str) {
        let Some(pathstring) = canonical_string(path)     // Option<PathBuf>
            .map(|pb| pb.into_os_string())               // Option<OsString>
            .and_then(|s| s.into_string().ok())          // Option<String>
        else {
            return;
        };

        // check if pathstring already exists:
        if let Some(data) = self.contents.get_mut(&pathstring) {
            // it's there, increment the rating.
            log_only(conf, &format!("Uprating path: {}", pathstring));
            data.update_with(&RasdfBaseData::new(&conf, Some(1.0), None, ""));
        } else {
            // new path, add it to the database
            log_only(conf, &format!("Adding new path: {}", pathstring));
            self.contents.insert(pathstring,
                RasdfBaseData::new(&conf, Some(1.0), None, ""));
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

    // add one row given as a string to self.contents; return new length of contents
    pub fn add_line(&mut self, conf: &Config, row: &str) -> usize {
        // ignore a blank line
        if row.is_empty() {
            return self.contents.len();
        }

        let mut v: Vec<&str> = row.split('|').collect();
        // if there are only three elements, add empty one at the end
        if v.len() == 3 {
            v.push("");
        }
        if v.len() != 4 {
            // not legal line
            log(conf, &format!("Can't parse row: {}", row));
            return self.contents.len();
        }

        // get a valid path string from v[0]
        let Some(pathstring) = canonical_string(v[0])      // Option<PathBuf>
                .map(|pb| pb.into_os_string())             // Option<OsString>
                .and_then(|s| s.into_string().ok())        // Option<String>
        else {
            log_only(conf, &format!("Cannot parse path: {}", v[0]));
            return self.contents.len();
        };

        // check the other three fields
        let (Ok(rating), Ok(date), flags) =
            (v[1].parse::<f32>(), v[2].parse::<u64>(), v[3].to_string())
        else {
            log_only(conf, &format!("Problem with fields: {}", row));
            return self.contents.len();
        };

        // all okay, insert the row.
        self.contents.insert(
            pathstring,
            RasdfBaseData {
                rating,
                date,
                flags,
            });

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

        // we want elements to look for either raw or lower-cased
        let elements: Vec<String> = match conf.case_sensitive {
            true => conf.arguments.clone(),
            false => conf.arguments.iter().map(|s| s.to_lowercase()).collect(),
        };

        // loop through all the paths in the data file
        'paths: for path in self.contents.keys() {
            // check if we're looking for dirs or folders (or both)
            let p = PathBuf::from(&path);
            if (!conf.find_dirs && p.is_dir()) || (!conf.find_files && p.is_file()) {
                continue 'paths;
            }

            // make a mutable copy to turn into lowercase.
            let mut pathstring = path.to_string();
            // and an index to move along it
            let mut start = 0usize;

            // if case insensitive, convert everything to lowercase
            if !conf.case_sensitive {
                pathstring = pathstring.to_lowercase();
            };

            // for each element provided by the user
            for element in elements.iter() {

                // look through the pathstring and find it
                if let Some(p) = pathstring[start..].find(element) {
                    start += p + element.len();
                } else {
                    continue 'paths;
                }
            }
            // only add the path if the last element was in the last segment.
            if !conf.strict {
                v.push(path);

            } else {
                // check out the last element, elements cannot be empty vec?
                let last_element = elements.last().unwrap();
                let last_segment_start = pathstring.rfind(MAIN_SEPARATOR).unwrap_or(0);
                if pathstring[last_segment_start..].find(last_element).is_some() {
                    v.push(path);
                }
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

fn canonical_string(path: &str) -> Option<PathBuf> {
    // return a Some(String) if path is a real path
    // otherwise None
    let mut pathstring = String::from(path);
    if pathstring.starts_with('~') {
        pathstring = pathstring.replacen('~', &home_dir()?, 1)
    }

    fs::canonicalize(&pathstring).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_string() {
        assert!( canonical_string("/home/tim/tmp").is_some());
        assert!( canonical_string("/home/user/tmp").is_none());
    }
}
