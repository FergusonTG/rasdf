#![allow(dead_code)]

use std::env;
use std::fs;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

const DATAFILE: &str = "/home/tim/.local/share/fasd/fasd.dat";

#[derive(Debug)]
pub struct AsdfBaseData {
    pub rating: f32,
    pub date: u64,
    pub flags: String,
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
        let contents = fs::read_to_string(&filename)
            .unwrap_or_else(|_| format!(
                    "Something went wrong reading \"{}\".", &filename
                    ));

        AsdfBase::from_data(&contents)
    }

    pub fn write_out(&self, filename: &str) -> std::io::Result<()> {
        let path = Path::new(&env::temp_dir()).join("myfile.txt");
        let mut file = fs::File::create(&path).unwrap();

        if let Err(e) = file.write(b"Brian was here. Briefly.\n") {
            println!("Could not write to file: {}\n", e);
            }

        fs::rename(&path, &filename).or(
            if let Err(e) = fs::copy(&path, &filename) { Err(e) } else { Ok(()) }
            )
    }

    pub fn find(&self, elements: &[&str]) -> Vec<&str> {
        let mut v = Vec::<&str>::new();

        'keys: for key in self.contents.keys() {
            let mut k = &key[..];
            for element in elements {
                if let Some(p) = key.find(element) {
                    if let Some(s) = k.get((p + element.len())..) {
                        k = s
                    } else {
                        k = ""
                    };
                } else {
                    continue 'keys;
                }
            }
            v.push(key);
        }
        v
    }
}

pub fn default_datafile() -> String {
    env::var("RASDF_DATAFILE").unwrap_or_else(|_| String::from(DATAFILE))
}
