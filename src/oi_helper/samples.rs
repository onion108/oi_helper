//! This file contains functions and data types for processing samples.

use std::{path::{Path, PathBuf}, fs::{File, OpenOptions}, io::{Read, Write}};

use json::{JsonValue, object};

use super::utils::web::{get_luogu_problem_content, get_test_case_from_luogu_tree};

pub struct Samples {
    config: JsonValue,
    config_file_path: String,
    iter_counter: usize,
}

#[allow(dead_code)]
pub struct SampleInfo {
    pub expected_in: String,
    pub expected_out: String,
    pub timeout: u32,
    pub memory_limit: u32,
    pub points: u32,
}

#[allow(dead_code)]
impl Samples {
    /// Construct from a file. `filename` is the path to the configuration file, i.e., `samples_info.json`
    pub fn from_file(filename: &str) -> Result<Self, Option<String>> {

        let path = Path::new(filename);
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                return Err(Some(format!("Cannot read configuration file: {}", err)));
            }
        };

        // Read the file
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Ok(_) => {}
            Err(err) => {
                return Err(Some(format!("Cannot read configuration file: {}", err)));
            }
        }
        
        // Convert the file content to json.
        let jsoned_content = match json::parse(&buffer) {
            Ok(obj) => obj,
            Err(err) => {
                return Err(Some(format!("Cannot read configuration file: {}", err)));
            }
        };

        Ok(Self { config: jsoned_content, config_file_path: String::from(filename), iter_counter: 0 })

    }

    fn get_default_config() -> JsonValue {
        object! {
            "sample_list": []
        }
    }

    /// Create a sample configuration.
    pub fn create(filename: &str) -> Result<(), Option<String>> {
        if crate::is_debug() {
            println!("[DEBUG] Creating with (filename = {filename})");
        }
        let path = Path::new(filename);
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(err) => {
                return Err(Some(format!("Cannot create configuration file: {}", err)));
            }
        };
        
        // Prepare the default content.
        let default_config = Self::get_default_config();

        match write!(&mut file, "{}", default_config.dump()) {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                Err(Some(format!("Cannot write to the configuration file: {}", err)))
            }
        }
    }

    /// Save the configuration.
    fn save(&self) -> Result<(), Option<String>> {
        let mut file = match OpenOptions::new().truncate(true).write(true).open(&Path::new(&self.config_file_path)) {
            Ok(file) => file,
            Err(err) => {
                return Err(Some(format!("Cannot open configuration file: {}", err)));
            }
        };
        match write!(&mut file, "{}", self.config.dump()) {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                return Err(Some(format!("Cannot write to the configuration file: {}", err)));
            }
        }
    }

    fn check_config(&self) -> Result<(), Option<String>> {
        if !(self.config.has_key("sample_list") && self.config["sample_list"].is_array()) {
            return Err(Some(format!("The configuration of the sample list is broken. Please check the samples_info.json.")))
        }
        Ok(())
    }

    /// Create a sample.
    pub fn create_sample(&mut self, points: u32, timeout: u32, mem_limit: u32) -> Result<i32, Option<String>> {
        self.check_config()?;
        let next_no = self.config["sample_list"].len();
        let parent_dir = match Path::new(&self.config_file_path).parent() {
            Some(p) => p,
            None => {
                unreachable!();
            }
        };

        let in_file_path_buf = parent_dir.join(&format!("{}.in", next_no));
        let out_file_path_buf = parent_dir.join(&format!("{}.out", next_no));

        match File::create(in_file_path_buf.as_path()) {
            Ok(_) => {}
            Err(err) => {
                return Err(Some(format!("Error creating sample #{next_no}: {err}")));
            }
        }

        match File::create(out_file_path_buf.as_path()) {
            Ok(_) => {}
            Err(err) => {
                return Err(Some(format!("Error creating sample #{next_no}: {err}")));
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
        self.save()?;

        Ok(next_no as i32)
    }

    fn read_from_pathbuf(&self, pthbuf: &PathBuf) -> Result<String, Option<String>> {
        let mut buffer = String::new();
        match File::open(pthbuf) {
            Ok(mut file) => {
                match file.read_to_string(&mut buffer) {
                    Ok(_) => Ok(buffer),
                    Err(err) => {
                        return Err(Some(format!("Error reading {}: {err}", pthbuf.to_str().unwrap())))
                    }
                }
            }
            Err(err) => {
                return Err(Some(format!("Error reading {}: {err}", pthbuf.to_str().unwrap())));
            }
        }
    }

    /// Get in-out for a sample with index.
    fn get(&self, idx: usize) -> Result<Option<SampleInfo>, Option<String>> {
        self.check_config()?;
        if idx >= self.config["sample_list"].len() {
            return Ok(None);
        }
        let infile_name = self.config["sample_list"][idx]["in_file"].to_string();
        let outfile_name = self.config["sample_list"][idx]["out_file"].to_string();

        // Concat paths
        let parent = Path::new(&self.config_file_path).parent().unwrap();
        let infile_pathbuf = parent.join(infile_name);
        let outfile_pathbuf = parent.join(outfile_name);
        
        // Read contents
        let infile_content = self.read_from_pathbuf(&infile_pathbuf)?;
        let outfile_content = self.read_from_pathbuf(&outfile_pathbuf)?;

        Ok(Some(SampleInfo {
            expected_in: infile_content,
            expected_out: outfile_content,
            timeout: match self.config["sample_list"][idx]["timeout_ms"].as_u32() {
                Some(timeout) => timeout,
                None => {
                    return Err(Some(format!("Error reading sample: invalid timeout value. ")));
                }
            },
            memory_limit: match self.config["sample_list"][idx]["memory_limit"].as_u32() {
                Some(timeout) => timeout,
                None => {
                    return Err(Some(format!("Error reading sample: invalid memory_limit value. ")));
                }
            },
            points: match self.config["sample_list"][idx]["points"].as_u32() {
                Some(timeout) => timeout,
                None => {
                    return Err(Some(format!("Error reading sample: invalid points value. ")));
                }
            },
        }))
    }

    /// Load the samples from Luogu with a specified problem id
    pub fn load_sample_from_luogu(&mut self, problem_id: &str) -> Result<(), Option<String>> {

        eprintln!("Starting fetching samples from {}... ", problem_id);

        let samples = get_test_case_from_luogu_tree(&(match get_luogu_problem_content(problem_id) {
            Ok(o) => o,
            Err(err) => return Err(Some(format!("Error occured while fetching samples: {}", err)))
        }));
        let each_point = 100 / (samples.len() as u32);

        eprintln!("Content fetched. Loading {} sample(s)... ", samples.len());

        for case in samples {

            let number = self.create_sample(each_point, 1000, 256)?;
            eprintln!("Loading sample #{number}... ");

            let in_path = Path::new(&self.config_file_path).parent().unwrap().join(&format!("{}.in", number));
            let out_path = Path::new(&self.config_file_path).parent().unwrap().join(&format!("{}.out", number));

            let mut in_file = match OpenOptions::new().write(true).truncate(true).open(&in_path) {
                Ok(f) => f,
                Err(err) => {
                    return Err(Some(format!("Error while opening sample from {}: {}", in_path.to_str().unwrap_or("undefined"), err)));
                }
            };

            let mut out_file = match OpenOptions::new().write(true).truncate(true).open(&out_path) {
                Ok(f) => f,
                Err(err) => {
                    return Err(Some(format!("Error while opening sample from {}: {}", out_path.to_str().unwrap_or("undefined"), err)));
                }
            };

            match write!(&mut in_file, "{}", case.0) {
                Ok(_) => {}
                Err(err) => {
                    return Err(Some(format!("Error while writing sample: {}", err)));
                }
            }

            match write!(&mut out_file, "{}", case.1) {
                Ok(_) => {}
                Err(err) => {
                    return Err(Some(format!("Error while writing sample: {}", err)));
                }
            }

            eprintln!("Loaded sample #{number}. ");

        }

        eprintln!("Fetching done. ");
        Ok(())

    }

}

impl Iterator for Samples {
    type Item = Result<SampleInfo, Option<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.get(self.iter_counter) {
            Ok(value) => match value {
                Some(value) => {
                    self.iter_counter += 1;
                    Some(Ok(value))
                },
                None => {
                    self.iter_counter = 0;
                    None
                }
            },
            Err(err) => Some(Err(err)),
        }
    }
}

