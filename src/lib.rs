#![allow(dead_code)]

use std::env;
use std::fs;

use std::collections::HashMap;
use std::io::Write;
use std::path::{PathBuf, MAIN_SEPARATOR};

pub mod config;
use config::{Config, ScoreMethod};

#[derive(Debug)]
pub struct AsdfBaseData {
    pub rating: f32,
    pub date: u64,
    pub flags: String,
}

impl AsdfBaseData {
    pub fn score(&self, conf: &Config) -> f32 {
        match conf.method {
            ScoreMethod::Date       => self.date as f32,
            ScoreMethod::Rating     => self.rating,
            ScoreMethod::Frecency => 
                self.rating * match conf.current_time - self.date {
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

    pub fn add(&mut self, conf: &Config, path: &str, flags: &str) {
        
        let path_result = fs::canonicalize(path);
        if path_result.is_err() { 
            // println!("Can't add {}", path);
            return; }

        let path = canonical_string(path);
        if path.is_empty() {
            // println!("Path is empty: {}", path);
            return; };

        if let Some(data) = self.contents.get_mut(&path) {
            // println!("Updating {}", path); 
            data.rating += 1.0 / data.rating;
            data.date = conf.current_time;
            for c in flags.chars() {
                if !data.flags.contains(c) {
                    data.flags.push(c);
                }
            }
        } else {
            // println!("Adding new path {}", path); 
            self.contents.insert(
                path,
                AsdfBaseData {
                    rating: 1.0,
                    date: conf.current_time,
                    flags: String::from(flags),
                },
            );
        }
    }

    pub fn remove(&mut self, row: &str) {
        if self.contents.remove(row).is_some() {
            // println!("Row removed: {}", row);
        } else {
            // println!("Could not find row to remove: {}", row);
        };
    }

    pub fn add_line(&mut self, row: &str) -> usize {
        // ignore a blank line
        if row.is_empty() { return self.contents.len(); }

        let mut v: Vec<&str> = row.split('|').collect();
        if v.len() == 3 {
            v.push("");
        }
        if v.len() == 4 {
            let path = canonical_string(v[0]);
            if path.is_empty() { self.contents.len(); }
            self.contents.insert(
                path,
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

    pub fn from_data(lines: &str) -> AsdfBase {
        let mut dbase = AsdfBase::new();
        for line in lines.split('\n') {
            dbase.add_line(line);
        }
        dbase
    }

    pub fn from_file(conf: &Config) -> AsdfBase {
        if let Ok(contents) = fs::read_to_string(&conf.datafile) {
            // eprintln!("{}", &contents);
            AsdfBase::from_data(&contents)
        } else {
            AsdfBase::new()
        }
    }

    pub fn clean(&mut self, conf: &Config) {

        if self.len() <= conf.maxlines { return; };

        for rec in self.contents.values_mut() {
            rec.rating *= 0.9;
        }

        let mut keys: Vec<_> = self.contents.keys().map(|f| (f.to_string(), self.contents[f].rating)).collect();
        keys.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        keys.truncate(keys.len() - conf.maxlines);

        for f in keys.iter() {
            self.contents.remove(&f.0);
        };
        return;
        
    }

    pub fn write_out(&self, conf: &Config) -> std::io::Result<()> {
        let path = env::temp_dir().join("myfile.txt");
        // println!("Writing to temp file {:?}", path);
        let mut file = fs::File::create(&path).unwrap();

        // write data out to temp file
        for (key, value) in &self.contents {
            file.write(format!("{}|{}|{}|{}\n", key, value.rating, value.date, value.flags).as_bytes())?;
        }

        // and copy that back to proper place
        // println!("Copying to {:?}", &conf.datafile);
        fs::rename(&path, &conf.datafile)
            .or(match fs::copy(&path, &conf.datafile) {
                Err(e) => Err(e),
                Ok(_)  => Ok(()),
            })
    }

    pub fn find_list(&self, conf: &Config, elements: &Vec<String>) -> Vec<(&str, f32)> {
        let mut v = Vec::<&str>::new();

        'paths: for path in self.contents.keys() {

            // should be unneccessary since all paths are canonicalized
            let pathstring = canonical_string(&path);
            if pathstring.is_empty() { continue 'paths };

            let p = PathBuf::from(&pathstring);
            if ! conf.find_dirs && p.is_dir() { continue 'paths }
            else if ! conf.find_files && p.is_file() { continue 'paths }

            let mut start = 0usize;
            for element in elements {
                if let Some(p) = path[start..].find(element) {
                    start += p + element.len();
                } else {
                    continue 'paths;
                }
            }
            // only add the path if the last element was in the last segment.
            if conf.strict == false || path[start..].find(MAIN_SEPARATOR).is_none() { 
                v.push(&path);
            }
        }
        // collect each path into a (path, score) tuple
        v.iter().map(|path| (*path, self.entry(path).unwrap().score(&conf))).collect()
    }

    pub fn find(&self, conf: &Config, elements: &Vec<String>) -> Option<&str> {
        let v  = self.find_list(&conf, elements);

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

fn canonical_string(path: &str) -> String {
    let p = PathBuf::from(path);

    match p.canonicalize() {
        Ok(pathbuf) => {
            match pathbuf.to_str() {
                Some(pathstring) => pathstring.to_string(),
                None => String::new(),
            }
        }
        _ => return String::new(),
    }
}
