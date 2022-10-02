//! This file contains operationgs and data structure about a workspace.

use std::{
    fs::{self, File, OpenOptions},
    io::{stdin, Read, Write},
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
    time::Duration,
};

use crossterm::style::Stylize;
use json::{object, JsonValue};
use wait_timeout::ChildExt;

use crate::oi_helper::utils;

use super::{resource, samples::Samples};

/// The workspace model.
pub struct Workspace {
    config: JsonValue,
    global_config: Option<String>,
}

#[allow(dead_code)]
impl Workspace {
    fn get_default_config(&self) -> JsonValue {
        // The default configuration, if the configuration doesn't exist.
        let builtin_default = object! {
            "initialzed": "true",
            "oi_helper_version": crate::VERSION,
            "cc_flags": "-std=c++11 -O2 -Wall -xc++ ",
            "cc_template": "temp0",
            "cc_default_extension": "cc",
            "cc_compiler": "g++",
        };

        // If the configuration directory exists
        if let Some(p) = &self.global_config {
            // Construct a path to the global.json
            let pth = Path::new(&p);
            let mut pth_buf = pth.to_path_buf();
            pth_buf.push("global.json");

            // Check if the file exists
            if !pth_buf.as_path().exists() {
                // If not, create a new file
                let mut f = File::create(&pth_buf.as_path()).unwrap();
                f.write_all(builtin_default.dump().as_bytes()).unwrap();
                builtin_default
            } else {
                // Otherwise, read the file OR update the default file.
                let mut f = File::open(&pth_buf.as_path()).unwrap();
                let mut buffer = String::new();
                f.read_to_string(&mut buffer).unwrap();
                let ccfg = json::parse(&buffer).unwrap();

                // Check the version
                if ccfg["oi_helper_version"].to_string() != crate::VERSION {
                    // The global.json is from an older version
                    let mut mccfg = ccfg.clone();

                    // Update keys that don't exist in the older version's configuration file
                    for i in builtin_default.entries().map(|x| x.0) {
                        if !mccfg.has_key(i) {
                            mccfg[i] = builtin_default[i].clone();
                        }
                    }

                    // Update the version.
                    mccfg["oi_helper_version"] = JsonValue::String(String::from(crate::VERSION));

                    let mut f = File::create(&pth_buf.as_path()).unwrap();
                    f.write_all(mccfg.dump().as_bytes()).unwrap();
                    mccfg
                } else {
                    // If it's already the newest, just return what we read
                    ccfg
                }
            }
        } else {
            builtin_default
        }
    }

    /// Edit the global configuration file.
    pub fn set_g_config(&mut self, key: &str, value: &str) {
        if let Some(p) = &self.global_config {
            let pth = Path::new(p);
            let mut pth_buf = pth.to_path_buf();
            pth_buf.push("global.json");
            let mut cfg = self.get_default_config();
            cfg[key] = JsonValue::String(String::from(value));
            let mut f = File::create(&pth_buf.as_path()).unwrap();
            f.write_all(cfg.dump().as_bytes()).unwrap();
        } else {
            eprintln!(
                "{}",
                "[Error] Cannot edit the global configuration file."
                    .bold()
                    .red()
            );
            exit(-1);
        }
    }

    /// Create from path.
    pub fn create(path: &PathBuf, global_cfg: &Option<String>) -> Self {
        let result = Self {
            config: JsonValue::Null,
            global_config: global_cfg.clone(),
        };
        let default_workspace_file = result.get_default_config();
        let mut cfg_path = path.clone();

        cfg_path.push("oi_ws.json");
        let cfg_file = cfg_path.as_path();

        // Check if the configuration exists.
        if !cfg_file.exists() {
            let mut f =
                match File::create(&cfg_file) {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("{}", format!("Cannot create workspace file: {}", err).bold().red());
                        exit(-1);
                    }
                };
            match f.write_all(default_workspace_file.dump().as_bytes()) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{}", format!("Cannot write to workspace file: {}", err).bold().red());
                    exit(-1);
                }
            }
        } else {

            // Let the user choose if they want to override the existed workspace file.
            eprintln!("{} The workspace configuration file already exists. Are you sure to override it? [Y/{}]", "[WARNING]".bold().yellow(), "N".bold().blue());
            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).unwrap();
            if choice.trim().to_uppercase() == "Y" {
                // User chose yes, then override it.
                let mut f =
                    File::create(&cfg_file).expect("cannot create the workspace file. stopped.");
                match f.write_all(default_workspace_file.dump().as_bytes()) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!(
                            "{}",
                            format!("Cannot write to the workspace file: {}", err)
                                .bold()
                                .red()
                        );
                        exit(-1);
                    }
                }
            } else {
                exit(-1);
            }
        }
        Self::from_json(default_workspace_file, global_cfg)
    }

    /// Initialize from json.
    pub fn from_json(json: JsonValue, global_cfg: &Option<String>) -> Self {
        Self {
            config: json.clone(),
            global_config: global_cfg.clone(),
        }
    }

    /// Initialize from file.
    pub fn from_file(path: &Path, global_cfg: &Option<String>) -> Self {
        // let mut file = File::open(path).expect("cannot find workspace config. stopped. \nHint: Have you executed `oi_helper init` or are you in the root directory of the workspace?");
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("{}", "Cannot find workspace's configuration file. Stopped. ".red());
                eprintln!("{}{}{}", "[HINT] Have you executed ".yellow().bold(), "oi_helper init".cyan().bold(), " or are you in the root directory of the workspace? ".bold().yellow());
                exit(-1);
            }
        };
        let mut file_content = String::new();
        match file.read_to_string(&mut file_content) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("cannot read workspace file: {}", err);
                eprintln!(
                    "{}{}{}",
                    "[HINT] Have you ran ".bold().yellow(),
                    "oi_helper init".bold().cyan(),
                    " yet? ".bold().yellow()
                );
                exit(-1);
            }
        }
        return Self::from_json(
            json::parse(&file_content).expect("the oi_ws.json is not a valid json file. stopped."),
            global_cfg,
        );
    }

    /// Check the version of the workspace.
    pub fn check_version(&mut self, p: &str) {
        // If have this key.
        if self.config.has_key("oi_helper_version") {
            // Get the version string.
            let version = self.config["oi_helper_version"].clone();

            // Check the version.
            if version.to_string() != crate::VERSION {
                eprintln!("{} The version of oi_helper is {} but the workspace version is {}. Load it anyway? [Y/{}]", "[WARNING]".bold().yellow(), crate::VERSION.bold().green(), version.to_string().bold().red(), "N".bold().blue());
                eprintln!("{}", "[HINT] You can use `oi_helper update` to update your workspace to the newest version safely.".bold().yellow());
                let mut u_c = String::new();
                stdin().read_line(&mut u_c).unwrap();

                // Unsafely update the workspace.
                if u_c.trim().to_uppercase() == "Y" {
                    self.config["oi_helper_version"] =
                        JsonValue::String(String::from(crate::VERSION));
                    self.config["__unsafe_updating"] = JsonValue::Boolean(true);
                    self.save_config(Path::new(p));
                } else {
                    // Just exit the program if the user didn't want to load the workspace.
                    // Maybe they'll update the workspace in a safe way later.
                    exit(-1);
                }
            } else {

                // Check if the workspace is unsafe.
                if self.config.has_key("__unsafe_updating") {
                    if let Some(uu) = self.config["__unsafe_updating"].as_bool() {
                        // It's true!
                        if uu {
                            eprintln!(
                                "{}",
                                "[WARNING] Running in an unsafe updated workspace. "
                                    .bold()
                                    .yellow()
                            );
                            eprintln!(
                                "{}{}{}",
                                "[HINT] Use ".bold().yellow(),
                                "oi_helper update".bold().cyan(),
                                " to update the workspace safely. ".bold().yellow()
                            )
                        }
                    }
                }
                // Otherwise, it's must be safe and the newest
            }
        } else {
            // Cannot get the version, which means it's broken.
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
    pub fn create_cpp(&self, name: &str, template: &str, maxn: &str, maxl: &str, debug_kit: bool) {
        let real_name = if name.ends_with(".cpp") && name.ends_with(".cc") && name.ends_with(".cxx")
        {
            String::from(name)
        } else {
            String::from(name) + "." + self.config["cc_default_extension"].to_string().as_str()
        };
        let mut file = match File::create(Path::new(&real_name)) {
            Ok(file) => file,
            Err(_) => {
                eprintln!(
                    "{}",
                    format!(
                        "Failed to create the C++ source file. Please check your configuration."
                    )
                    .bold()
                    .red()
                );
                exit(-1);
            }
        };
        let template_scheme_obj = self.config["cc_template"].to_string();
        let mut buffer = String::new();
        let template_scheme = template_scheme_obj.as_str();

        // Choose the tempalte.
        let template = match template {
            "dp" => match template_scheme {
                "temp1" => resource::CPP_DP_TEMPLATE_0.trim_start(),
                "temp0" | _ => resource::CPP_DP_TEMPLATE_1.trim_end(),
            },
            "default" => match template_scheme {
                "temp1" => resource::CPP_TEMPLATE_1.trim_start(),
                "temp0" | _ => resource::CPP_TEMPLATE_0.trim_start(),
            },
            "dp-2d" => match template_scheme {
                "temp1" => resource::CPP_DP_2D_TEMPLATE_1.trim_start(),
                "temp0" | _ => resource::CPP_DP_2D_TEMPLATE_0.trim_start(),
            },
            "empty" => "",
            _ => {
                // Try to treat the template name as a path to the template file.
                if let Ok(mut f) = File::open(&Path::new(template)) {
                    // Try to read it.
                    match f.read_to_string(&mut buffer) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!(
                                "Failed to read template file {} due to error: {}",
                                template, err
                            );
                            exit(-1);
                        }
                    }
                    buffer.as_str()
                } else {
                    // All tryings were failed. Tell the user about that.
                    eprintln!("Invalid template: {}", template);
                    eprintln!(
                        "{}",
                        "Usable tempaltes: dp, default, dp-2d, empty, [path/to/template]"
                            .bold()
                            .yellow()
                    );
                    exit(-1);
                }
            }
        };

        // Fill in all the placeholders and write.
        file.write_all(
            template
                .replace("{##}", name)
                .replace("{#maxn_value#}", maxn)
                .replace("{#debug_kit#}", {
                    if debug_kit {
                        resource::CPP_TEMPLATE_DEBUG_KIT
                    } else {
                        ""
                    }
                })
                .replace("{#maxl_value#}", maxl)
                .as_bytes(),
        )
        .unwrap();
    }

    fn compile_cpp(&self, real_name: &str, executable_name: &str, use_debug: bool) {
        match Command::new(self.config["cc_compiler"].to_string().as_str())
            .args(self.parse_args())
            .arg(format!("-o"))
            .arg(format!("{}", executable_name))
            .arg({
                if use_debug {
                    "-D__DEBUG__"
                } else {
                    ""
                }
            })
            .arg("--")
            .arg(real_name)
            .status()
        {
            Ok(_) => {
                println!("{}", "Compiled. ".bold().green());
            }
            Err(_) => {
                eprintln!("Failed to compile the program. Stopped. (CE(0))");
                exit(-1);
            }
        }
    }

    /// Run a C++ source file.
    pub fn run_cpp(&self, name: &str, use_debug: bool) {
        // Get the real name.
        let real_name = if name.ends_with(".cpp") && name.ends_with(".cc") && name.ends_with(".cxx")
        {
            String::from(name)
        } else {
            String::from(name) + "." + self.config["cc_default_extension"].to_string().as_str()
        };

        // Generate the executable's name.
        let executable_name = real_name.split('.').collect::<Vec<&str>>()[0];

        // Check if the old build target is already built and haven't been removed yet.
        if Path::new(executable_name).exists() {
            match fs::remove_file(Path::new(executable_name)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!("failed to clean the old built target:{}", err)
                            .bold()
                            .red()
                    );
                    exit(-1);
                }
            }
        }

        // Compile the target.
        self.compile_cpp(&real_name, executable_name, use_debug);

        // Run the target.
        match Command::new(format!("./{}", executable_name)).status() {
            Ok(_) => {}
            Err(_) => {
                eprintln!("{}", format!("Runtime error occurred. ").bold().red());
            }
        }

        // Finally remove the file.
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
                    buffer.push(i);
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
            println!(
                "Current Workspace's OI Helper Version (oi_helper_version): {} {}",
                self.config["oi_helper_version"].to_string(),
                if self.config.has_key("__unsafe_updating") {
                    if let Some(uu) = self.config["__unsafe_updating"].as_bool() {
                        if uu {
                            "UNSAFE UPDATED".yellow().bold()
                        } else {
                            "".stylize()
                        }
                    } else {
                        "MAYBE BROKEN".red().bold()
                    }
                } else {
                    "".reset()
                }
            );
        }
        if self.config.has_key("cc_flags") {
            println!(
                "Current C++ Compiler Flags (cc_flags): {}",
                self.config["cc_flags"].to_string()
            );
        }
        if self.config.has_key("cc_template") {
            println!(
                "Current Template Theme (cc_template): {}",
                self.config["cc_template"].to_string()
            );
        }
        if self.config.has_key("cc_default_extension") {
            println!(
                "Current C++ Extension (cc_default_extension): {}",
                self.config["cc_default_extension"].to_string()
            );
        }
        if self.config.has_key("cc_compiler") {
            println!(
                "Current C++ Compiler (cc_compiler): {}",
                self.config["cc_compiler"].to_string()
            );
        }
    }

    /// Update the workspace file to the newest version.
    pub fn update(&mut self) {
        if self.config.has_key("oi_helper_version") {
            self.config["oi_helper_version"] = JsonValue::String(String::from(crate::VERSION));
        }
        let default = &self.get_default_config();
        for i in default.entries().map(|x| x.0) {
            if !self.config.has_key(i) {
                self.config[i] = default[i].clone();
            }
        }
        self.config["__unsafe_updating"] = JsonValue::from(false);
    }

    /// Test the given target.
    pub fn test(&self, name: &str, sample_group: &mut Samples) {
        // Get the real name.
        let real_name = if name.ends_with(".cpp") && name.ends_with(".cc") && name.ends_with(".cxx")
        {
            String::from(name)
        } else {
            String::from(name) + "." + self.config["cc_default_extension"].to_string().as_str()
        };

        // Generate the executable's name.
        let executable_name = real_name.split('.').collect::<Vec<&str>>()[0];

        // Check if the old build target is already built and haven't been removed yet.
        if Path::new(executable_name).exists() {
            match fs::remove_file(Path::new(executable_name)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!("failed to clean the old built target:{}", err)
                            .bold()
                            .red()
                    );
                    exit(-1);
                }
            }
        }

        // Compile the target.
        self.compile_cpp(&real_name, executable_name, false);

        // Run the tests
        let mut total_points = 0_u32;
        let mut group_id = 0;
        let temp_in = Path::new("tkejhowiuyoiuwoiub_in.bakabaka.in.txt");
        // Iterates over each test cases
        for i in sample_group {
            eprintln!("Testing test #{group_id}...");
            let timeout = Duration::from_millis(i.timeout as u64);
            let points = i.points;

            let mut in_file = match OpenOptions::new()
                .write(true)
                .read(true)
                .truncate(true)
                .create(true)
                .open(temp_in)
            {
                Ok(file) => file,
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!("Error running sample group #{group_id}: {err}")
                            .bold()
                            .red()
                    );
                    exit(-1);
                }
            };
            match write!(in_file, "{}", i.expected_in) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!("Error running sample group #{group_id}: {err}")
                            .bold()
                            .red()
                    );
                    exit(-1);
                }
            }
            if crate::is_debug() {
                println!(
                    "[DEBUG] Write {} to {}.",
                    i.expected_in,
                    temp_in.to_str().unwrap()
                );
            }

            in_file = match File::open(temp_in) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!("Error running sample group #{group_id}: {err}")
                    );
                    exit(-1);
                }
            };

            // Spawn the child process.
            let mut child = match Command::new(format!("./{}", executable_name))
                .stdin(in_file)
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(err) => {
                    eprintln!(
                        "{}",
                        format!("Error running sample group #{}: {err}", group_id)
                            .bold()
                            .red()
                    );
                    exit(-1);
                }
            };
            match child.wait_timeout(timeout) {
                Ok(is_timeout) => {
                    match is_timeout {

                        // Didn't time out.
                        Some(_) => {
                            // Read the result output.
                            let mut _tmp0 = child.wait_with_output().unwrap();
                            let content = String::from_utf8_lossy(&_tmp0.stdout[..]);

                            // Check and compare the results.
                            if content.trim() == i.expected_out {
                                eprintln!(
                                    "{}",
                                    format!("Test #{group_id} passed: AC({})", i.points).green()
                                );
                                total_points += points;
                            } else {
                                let colored_diffs =
                                    utils::strdiff::colored_diff(&i.expected_out, content.trim());
                                eprintln!("{}", format!("Test #{group_id} failed: WA(0)").red());
                                eprintln!("");
                                eprintln!("Expected: ");
                                // eprintln!("{}", i.expected_out.on_black());
                                for i in colored_diffs.0 {
                                    eprint!("{}", i);
                                }
                                eprintln!();
                                eprintln!("Actually: ");
                                // eprintln!("{}", content.trim().on_red());
                                for i in colored_diffs.1 {
                                    eprint!("{}", i);
                                }
                                eprintln!();
                                eprintln!("================================================");
                                eprintln!("Sample in: ");
                                eprintln!("{}", i.expected_in);
                            }
                        }

                        // Timeout.
                        None => {
                            child.kill().unwrap();
                            eprintln!("{}", format!("Test #{group_id} failed: TLE(0)").red());
                        }
                    }
                }
                Err(_) => {
                    child.kill().unwrap();
                    eprintln!("{}", format!("Test #{group_id} failed: TLE(0)").red());
                }
            }
            group_id += 1;
        }

        println!("Total points you get: {}", total_points);

        // Finally remove the files.
        fs::remove_file(Path::new(&format!("./{}", executable_name))).unwrap();
        fs::remove_file(temp_in).unwrap();
    }
}
