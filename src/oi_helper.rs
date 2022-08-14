//! The main model for OI Helper.

use std::{path::Path, fs};

use crate::OIHelperCommands;

use self::{workspace::Workspace, samples::Samples};

mod workspace;
mod resource;
mod samples;
mod samples_cli;
mod utils;


/// The main model of the OI Helper.
pub struct OIHelper {
    /// Commandline arguments
    args: crate::OIHelperCli,

    /// The global configuration file's path.
    global_config_path: Option<String>,
}

impl OIHelper {
    pub fn new(args: crate::OIHelperCli) -> Self {
        Self { args, global_config_path: None }
    }

    pub fn config(&mut self) {
        if let Some(user_home) = home::home_dir() {
            let mut p = user_home.clone();
            p.push(".oi_helper");
            if !p.as_path().exists() {
                fs::create_dir(&p).unwrap();
            }
            let ps = p.to_str();
            self.global_config_path = Some(String::from(ps.unwrap()));
        } else {
            self.global_config_path = None;
        }
    }
    
    pub fn run(&mut self) {
        match &self.args.subcommand {

            OIHelperCommands::Init { path } => {
                Workspace::create(&path, &self.global_config_path.clone());
            },

            OIHelperCommands::Checkver { path } => {
                let mut cfg_path = path.clone();
                cfg_path.push("oi_ws.json");
                Workspace::from_file(&cfg_path.as_path(), &self.global_config_path.clone()).check_version("./oi_ws.json");
            },

            OIHelperCommands::Config { key, value } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.set_config(key, value);
                workspace.save_config(Path::new("./oi_ws.json"));
            },

            OIHelperCommands::Create { name , template , maxn , maxl, debug_kit } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.check_version("./oi_ws.json");
                workspace.create_cpp(name, template, maxn, maxl, *debug_kit);
            },

            OIHelperCommands::Run { name, debug } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.check_version("./oi_ws.json");
                workspace.run_cpp(name, *debug);
            },

            OIHelperCommands::Info => {
                let workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.display_info();
            },

            OIHelperCommands::Update => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.update();
                workspace.save_config(&Path::new("./oi_ws.json"));
            },

            OIHelperCommands::GlobalConfig { key, value } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.set_g_config(key, value);
            },

            OIHelperCommands::Samples { subcommand } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.check_version("./oi_ws.json");
                samples_cli::samples(&mut workspace, subcommand)
            },

            OIHelperCommands::Test { target } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone());
                workspace.check_version("./oi_ws.json");
                let path_to_sampledir_str = format!("./{}.smpd", target.to_owned());
                let path_to_sampledir = Path::new(&path_to_sampledir_str);
                let mut samples = Samples::from_file(path_to_sampledir.join("samples_info.json").to_str().unwrap());
                workspace.test(target, &mut samples);
            }

        }
    }
}
