#![allow(dead_code)]

use std::env;
use std::fs;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, MAIN_SEPARATOR};
use std::time::SystemTime;

const DATAFILE: &str = "/home/tim/.local/share/fasd/fasd.dat";

#[derive(Copy, Clone, Debug)]
pub enum ScoreMethod { Date, Rating, Frecency }

#[derive(Debug)]
pub struct AsdfBaseData {
    pub rating: f32,
    pub date: u64,
    pub flags: String,
}

impl AsdfBaseData {
    pub fn score(&self, method: ScoreMethod, now: u64) -> f32 {
        match method {
            ScoreMethod::Date       => self.date as f32,
            ScoreMethod::Rating     => self.rating,
            ScoreMethod::Frecency => 
                self.rating * match now - self.date {
                    0 ..= 3600       => 6.0,
                    3601 ..= 86400   => 4.0,
                    86401 ..= 604800 => 2.0,
                    _                => 1.0,
                },
        }
    }
}


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
    pub fn len(&self) -> usize {
        self.contents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn entry(&self, path: &str) -> Option<&AsdfBaseData> {
        self.contents.get(path)
    }

    pub fn add_line(&mut self, row: &str) -> usize {
        if row.is_empty() { return self.contents.len(); }

        let mut v: Vec<&str> = row.split('|').collect();
        if v.len() == 3 {
            v.push("");
        }
        if v.len() == 4 {
            self.contents.insert(
                v[0].to_string(),
                AsdfBaseData {
                    rating: v[1].parse::<f32>().unwrap(),
                    date: v[2].parse::<u64>().unwrap(),
                    flags: v[3].to_string(),
                },
            );
        } else {
            panic!("Not a valid row: `{}`", row);
        }

        self.contents.len()
    }

    pub fn add(&mut self, path: &str, flags: &str) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(data) = self.contents.get_mut(path) {
            data.rating += 1.0;
            data.date = now;
            for c in flags.chars() {
                if !data.flags.contains(c) {
                    data.flags.push(c);
                }
            }
        } else {
            self.contents.insert(
                String::from(path),
                AsdfBaseData {
                    rating: 1.0,
                    date: now,
                    flags: String::from(flags),
                },
            );
        }
    }

    pub fn from_data(lines: &str) -> AsdfBase {
        let mut dbase = AsdfBase::new();
        for line in lines.split('\n') {
            dbase.add_line(line);
        }
        dbase
    }

    pub fn from_file(filename: &str) -> AsdfBase {
        if let Ok(contents) = fs::read_to_string(&filename) {
            AsdfBase::from_data(&contents)
        } else {
            AsdfBase::new()
        }
    }

    pub fn write_out(&self, filename: &str) -> std::io::Result<()> {
        let path = Path::new(&env::temp_dir()).join("myfile.txt");
        let mut file = fs::File::create(&path).unwrap();

        // write data out to temp file
        for (key, value) in &self.contents {
            file.write(format!("{}|{}|{}|{}\n", key, value.rating, value.date, value.flags).as_bytes())?;
        }

        // and copy that back to proper place
        fs::rename(&path, &filename)
            .or (match fs::copy(&path, &filename) {
                Err(e) => Err(e),
                Ok(_)  => Ok(()),
            })
    }

    pub fn find_list(&self, method: ScoreMethod, elements: &Vec<String>) -> Vec<(&str, f32)> {
        let mut v = Vec::<&str>::new();
        let now = current_timestamp();

        'paths: for path in self.contents.keys() {
            let mut k = &path[..];
            for element in elements {
                if let Some(p) = path.find(element) {
                    k = if let Some(s) = k.get((p + element.len())..) {
                        s
                    } else {
                        ""
                    };
                } else {
                    continue 'paths;
                }
            }
            // only add the path if the last element was in the last segment.
            if let None = k.find(MAIN_SEPARATOR) { 
                v.push(path) 
            };
        }
        v.iter().map(|path| (*path, self.entry(path).unwrap().score(method, now))).collect()
    }

    #[allow(unused_variables)]
    pub fn find(&self, method: ScoreMethod, elements: &Vec<String>) -> Option<&str> {
        let v  = self.find_list(method, elements);

        let mut ret:(Option<&str>, f32) = (None, f32::NAN);
        for t in v {
            if ret.0.is_none() {
                ret = (Some(t.0), t.1);
            } else {
                if ret.1.lt(&t.1) {
                    ret = (Some(t.0), t.1);
                }
            }
        }
        ret.0
    }
}

pub fn default_datafile() -> String {
    env::var("RASDF_DATAFILE").unwrap_or_else(|_| String::from(DATAFILE))
}

pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
}

