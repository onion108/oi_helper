//! This file contains operationgs and data structure about a workspace.

use std::{
    fs::{self, File, OpenOptions},
    io::{stdin, Read, Write},
    path::{Path, PathBuf},
    process::{exit, Command},
};

use crossterm::style::Stylize;
use json::{object, JsonValue};

use super::resource;

/// The workspace model.
pub struct Workspace {
    config: JsonValue,
}

#[allow(dead_code)]
impl Workspace {

    fn get_default_config() -> JsonValue {
        object! {
            "initialzed": "true",
            "oi_helper_version": crate::VERSION,
            "cc_flags": "-std=c++11 -O2 -Wall -xc++ ",
            "cc_template": "temp0",
            "cc_default_extension": "cc",
            "cc_compiler": "g++", 
        }
    }

    /// Create from path.
    pub fn create(path: &PathBuf) -> Self {
        let default_workspace_file = Self::get_default_config();
        let mut cfg_path = path.clone();

        cfg_path.push("oi_ws.json");
        let cfg_file = cfg_path.as_path();

        if !cfg_file.exists() {
            let mut f =
                File::create(&cfg_file).expect("cannot create the workspace file. stopped.");
            f.write_all(default_workspace_file.dump().as_bytes())
                .expect("cannot write to workspace file. stopped.");
        } else {
            eprintln!("{} The workspace configuration file already exists. Are you sure to override it? [Y/{}]", "[WARNING]".bold().yellow(), "N".bold().blue());
            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).unwrap();
            if choice.trim().to_uppercase() == "Y" {
                // User chose yes, then override it.
                let mut f =
                    File::create(&cfg_file).expect("cannot create the workspace file. stopped.");
                f.write_all(default_workspace_file.dump().as_bytes())
                    .expect("cannot write to workspace file. stopped.");
            } else {
                exit(-1);
            }
        }
        Self::from_json(default_workspace_file)
    }

    /// Initialize from json.
    pub fn from_json(json: JsonValue) -> Self {
        Self {
            config: json.clone(),
        }
    }

    /// Initialize from file.
    pub fn from_file(path: &Path) -> Self {
        let mut file = File::open(path).expect("cannot find workspace config. stopped. \nHint: Have you executed `oi_helper init` or are you in the root directory of the workspace?");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)
            .expect("cannot read from the workspace configuration file. stopped.");
        return Self::from_json(
            json::parse(&file_content).expect("the oi_ws.json is not a valid json file. stopped."),
        );
    }

    /// Check the version of the workspace.
    pub fn check_version(&mut self, p: &str) {
        if self.config.has_key("oi_helper_version") {
            let version = self.config["oi_helper_version"].clone();
            if version.to_string() != crate::VERSION {
                eprintln!("{} The version of oi_helper is {} but the workspace version is {}. Load it anyway? [Y/{}]", "[WARNING]".bold().yellow(), crate::VERSION.bold().green(), version.to_string().bold().red(), "N".bold().blue());
                eprintln!("{}", "[HINT] You can use `oi_helper update` to update your workspace to the newest version safely.".bold().yellow());
                let mut u_c = String::new();
                stdin().read_line(&mut u_c).unwrap();
                if u_c.trim().to_uppercase() == "Y" {
                    self.config["oi_helper_version"] =
                        JsonValue::String(String::from(crate::VERSION));
                    self.save_config(Path::new(p));
                } else {
                    exit(-1);
                }
            }
        } else {
            eprintln!(
                "{} The workspace config is broken or not in the correct format. Stopped.",
                "[ERROR]".red().bold()
            );
            exit(-1);
        }
    }

    /// Set the configuration.
    pub fn set_config(&mut self, key: &str, value: &str) {
        self.config[key] = JsonValue::String(String::from(value));
    }

    /// Save the configuration.
    pub fn save_config(&self, path: &Path) {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .expect("error: unable to save workspace config");
        file.write_all(self.config.dump().as_bytes()).unwrap();
    }

    /// Create a new C++ source file.
    pub fn create_cpp(&self, name: &str, template: &str) {
        let real_name = if name.ends_with(".cpp") && name.ends_with(".cc") && name.ends_with(".cxx")
        {
            String::from(name)
        } else {
            String::from(name) + "." + self.config["cc_default_extension"].to_string().as_str()
        };
        let mut file =
            File::create(Path::new(&real_name)).expect("error: cannot create cpp file. stopped. ");
        let template_scheme_obj = self.config["cc_template"].to_string();
        let template_scheme = template_scheme_obj.as_str();
        let template = match template {
            "dp" => match template_scheme {
                "temp1" => resource::CPP_DP_TEMPLATE_0.trim_start(),
                "temp0" | _ => resource::CPP_DP_TEMPLATE_1.trim_end(),
            }
            "default" | _ => match template_scheme {
                "temp1" => resource::CPP_TEMPLATE_1.trim_start(),
                "temp0" | _ => resource::CPP_TEMPLATE_0.trim_start(),
            }
        };
        file.write_all(template.replace("{##}", name).as_bytes())
            .unwrap();
    }

    /// Run a C++ source file.
    pub fn run_cpp(&self, name: &str) {
        let real_name = if name.ends_with(".cpp") && name.ends_with(".cc") && name.ends_with(".cxx")
        {
            String::from(name)
        } else {
            String::from(name) + "." + self.config["cc_default_extension"].to_string().as_str()
        };
        let executable_name = real_name.split('.').collect::<Vec<&str>>()[0];
        match Command::new(self.config["cc_compiler"].to_string().as_str())
            .args(self.parse_args())
            .arg(format!("-o"))
            .arg(format!("{}", executable_name))
            .arg("--")
            .arg(&real_name)
            .status()
        {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Failed to compile the program. Stopped. (CE(0))");
            }
        }
        match Command::new(format!("./{}", executable_name)).status() {
            Ok(_) => {}
            Err(_) => {
                eprintln!("(RE(0))");
            }
        }
        fs::remove_file(Path::new(&format!("./{}", executable_name))).unwrap();
    }

    fn parse_args(&self) -> Vec<String> {
        let mut result = Vec::<String>::new();
        let mut buffer = String::new();
        let source = self.config["cc_flags"].to_string();
        let mut status = 0;
        for i in source.chars() {
            match status {
                0 => {
                    match i {
                        ' ' => status = 1,
                        '"' => status = 2,
                        _ => buffer.push(i),
                    };
                }
                1 => {
                    if buffer != "" {
                        result.push(buffer);
                        buffer = String::new();
                        if i != ' ' {
                            buffer.push(i);
                        }
                    } else {
                        if i != ' ' {
                            buffer.push(i);
                        }
                    }
                    status = 0
                }
                2 => match i {
                    '"' => status = 0,
                    '\\' => status = 3,
                    _ => buffer.push(i),
                },
                3 => {
                    status = 0;
                }
                _ => {}
            }
        }
        result
    }

    /// Display the info of a workspace.
    pub fn display_info(&self) {
        if self.config.has_key("oi_helper_version") {
            println!("Current Workspace's OI Helper Version (oi_helper_version): {}", self.config["oi_helper_version"].to_string());
        }
        if self.config.has_key("cc_flags") {
            println!("Current C++ Compiler Flags (cc_flags): {}", self.config["cc_flags"].to_string());
        }
        if self.config.has_key("cc_template") {
            println!("Current Template Theme (cc_template): {}", self.config["cc_template"].to_string());
        }
        if self.config.has_key("cc_default_extension") {
            println!("Current C++ Extension (cc_default_extension): {}", self.config["cc_default_extension"].to_string());
        }
        if self.config.has_key("cc_compiler") {
            println!("Current C++ Compiler (cc_compiler): {}", self.config["cc_compiler"].to_string());
        }
    }

    /// Update the workspace file to the newest version.
    pub fn update(&mut self) {
        if self.config.has_key("oi_helper_version") {
            self.config["oi_helper_version"] = JsonValue::String(String::from(crate::VERSION));
        }
        let default = &Self::get_default_config();
        for i in default.entries().map(|x| x.0) {
            if !self.config.has_key(i) {
                self.config[i] = default[i].clone();
            }
        }
    }
}
