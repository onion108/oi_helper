//! This file contains functions and data types for processing samples.

use std::{path::{Path, PathBuf}, fs::{File, OpenOptions}, process::exit, io::{Read, Write}};

use crossterm::style::Stylize;
use json::{JsonValue, object};

pub struct Samples {
    config: JsonValue,
    config_file_path: String,
    iter_counter: usize,
}

#[allow(dead_code)]
pub struct SampleInfo {
    expected_in: String,
    expected_out: String,
    timeout: u32,
    memory_limit: u32,
    points: u32,
}

#[allow(dead_code)]
impl Samples {
    /// Construct from a file. `filename` is the path to the configuration file, i.e., `samples_info.json`
    pub fn from_file(filename: &str) -> Self {

        let path = Path::new(filename);
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{}", format!("Cannot read configuration file: {}", err).bold().red());
                exit(-1);
            }
        };

        // Read the file
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format!("Cannot read configuration file: {}", err).bold().red());
                exit(-1);
            }
        }
        
        // Convert the file content to json.
        let jsoned_content = match json::parse(&buffer) {
            Ok(obj) => obj,
            Err(err) => {
                eprintln!("{}", format!("Cannot read configuration file: {}", err).bold().red());
                exit(-1);
            }
        };

        Self { config: jsoned_content, config_file_path: String::from(filename), iter_counter: 0 }

    }

    fn get_default_config() -> JsonValue {
        object! {
            "sample_list": []
        }
    }

    /// Create a sample configuration.
    pub fn create(filename: &str) {
        let path = Path::new(filename);
        let mut file = match OpenOptions::new().create(true).truncate(true).open(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{}", format!("Cannot create configuration file: {}", err).bold().red());
                exit(-1);
            }
        };
        
        // Prepare the default content.
        let default_config = Self::get_default_config();

        match write!(&mut file, "{}", default_config.dump()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format!("Cannot write to the configuration file: {}", err).bold().red());
                exit(-1);
            }
        }
    }

    /// Save the configuration.
    fn save(&self) {
        let mut file = match OpenOptions::new().truncate(true).write(true).open(&Path::new(&self.config_file_path)) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{}", format!("Cannot open configuration file: {}", err).bold().red());
                exit(-1);
            }
        };
        match write!(&mut file, "{}", self.config.dump()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format!("Cannot write to the configuration file: {}", err).bold().red());
                exit(-1);
            }
        }
    }

    fn check_config(&self) {
        if !(self.config.has_key("sample_list") && self.config["sample_list"].is_array()) {
            eprintln!("{}", format!("The configuration of the sample list is broken. Please check the samples_info.json.").red().bold());
            exit(-1);
        }
    }

    /// Create a sample.
    pub fn create_sample(&mut self, points: u32, timeout: u32, mem_limit: u32) {
        self.check_config();
        let next_no = self.config["sample_list"].len();
        let parent_dir = match Path::new(&self.config_file_path).parent() {
            Some(p) => p,
            None => {
                unreachable!();
            }
        };

        // TODO: Create sample files.
        let in_file_path_buf = parent_dir.join(&format!("{}.in", next_no));
        let out_file_path_buf = parent_dir.join(&format!("{}.out", next_no));

        match File::create(in_file_path_buf.as_path()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format!("Error creating sample #{next_no}: {err}").bold().red());
                exit(-1);
            }
        }

        match File::create(out_file_path_buf.as_path()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", format!("Error creating sample #{next_no}: {err}").bold().red());
                exit(-1);
            }
        }

        // Register the sample info.
        // There should NOT panic because we've confirmed that the key sample_list is already an array.
        // So no need to process the error, just unwrap.
        self.config["sample_list"].push(object! {
            "in_file": format!("{next_no}.in"),
            "out_file": format!("{next_no}.out"),
            "timeout_ms": timeout,
            "memory_limit": mem_limit,
            "points": points,
        }).unwrap();

        // Save the configuration.
        self.save();
    }

    fn read_from_pathbuf(&self, pthbuf: &PathBuf) -> String {
        let mut buffer = String::new();
        match File::open(pthbuf) {
            Ok(mut file) => {
                match file.read_to_string(&mut buffer) {
                    Ok(_) => buffer,
                    Err(err) => {
                        eprintln!("Error reading {}: {err}", pthbuf.to_str().unwrap());
                        exit(-1);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error opening {}: {err}", pthbuf.to_str().unwrap());
                exit(-1);
            }
        }
    }

    /// Get in-out for a sample with index.
    fn get(&self, idx: usize) -> Option<SampleInfo> {
        self.check_config();
        if idx >= self.config["sample_list"].len() {
            return None;
        }
        let infile_name = self.config["sample_list"][idx]["in_file"].to_string();
        let outfile_name = self.config["sample_list"][idx]["out_file"].to_string();

        // Concat paths
        let parent = Path::new(&self.config_file_path).parent().unwrap();
        let infile_pathbuf = parent.join(infile_name);
        let outfile_pathbuf = parent.join(outfile_name);
        
        // Read contents
        let infile_content = self.read_from_pathbuf(&infile_pathbuf);
        let outfile_content = self.read_from_pathbuf(&outfile_pathbuf);

        Some(SampleInfo {
            expected_in: infile_content,
            expected_out: outfile_content,
            timeout: match self.config["sample_list"][idx]["timeout"].as_u32() {
                Some(timeout) => timeout,
                None => {
                    eprintln!("{}", format!("Error reading sample. ").bold().red());
                    exit(-1);
                }
            },
            memory_limit: match self.config["sample_list"][idx]["memory_limit"].as_u32() {
                Some(timeout) => timeout,
                None => {
                    eprintln!("{}", format!("Error reading sample. ").bold().red());
                    exit(-1);
                }
            },
            points: match self.config["sample_list"][idx]["points"].as_u32() {
                Some(timeout) => timeout,
                None => {
                    eprintln!("{}", format!("Error reading sample. ").bold().red());
                    exit(-1);
                }
            },
        })
    }

}

impl Iterator for Samples {
    type Item = SampleInfo;

    fn next(&mut self) -> Option<Self::Item> {
        match self.get(self.iter_counter) {
            Some(value) => {
                self.iter_counter += 1;
                Some(value)
            },
            None => {
                self.iter_counter = 0;
                None
            }
        }
    }
}
