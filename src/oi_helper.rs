//! The main model for OI Helper.

use std::{path::Path, fs::{self, OpenOptions}, io::{Write, Read, Error}};

use json::object;
use localoco::{strings::Strings, util::load_strings};

use crate::{OIHelperCommands, is_debug};

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


    /// The global localization.
    global_strings: Option<Strings>,
}

impl OIHelper {
    pub fn new(args: crate::OIHelperCli) -> Self {
        Self { args, global_config_path: None, global_strings: None }
    }

    pub fn config(&mut self) -> Result<(), anyhow::Error> {
        if let Some(user_home) = home::home_dir() {
            let mut p = user_home.clone();
            p.push(".oi_helper");
            if !p.as_path().exists() {
                fs::create_dir(&p).unwrap();
            }
            let ps = p.to_str();
            self.global_config_path = Some(String::from(ps.unwrap()));
            let mut op = p.clone();
            p.push("preference.json");
            let language;
            if !p.as_path().exists() {
                let mut f = OpenOptions::new().create(true).write(true).open(p)?;
                write!(f, "{}", ((object! {
                    "lang": "en_US",
                }).dump()))?;
                language = String::from("en_US");
            } else {
                let mut f = OpenOptions::new().read(true).open(p)?;
                let mut content_buffer = String::new();
                f.read_to_string(&mut content_buffer)?;
                let obj = json::parse(&content_buffer)?;
                if obj["lang"].is_string() {
                    // Load the strings file by the language
                    language = obj["lang"].to_string();
                } else {
                    language = String::from("en_US"); // Default loads en_US
                }
            }
            op.push("lang");
            self.global_strings = Some(load_strings(op, &language)?);
        } else {
            self.global_config_path = None;
            return Err(Error::new(std::io::ErrorKind::Other, "Cannot find global config directory! ").into());
        }
        if is_debug() {
            println!("{}", self.global_strings.as_ref().unwrap().translate("other.test"))
        }
        Ok(())
    }
    
    pub fn run(&mut self) -> Result<(), Option<String>> {
        match &self.args.subcommand {

            OIHelperCommands::Init { path } => {
                Workspace::create(&path, &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
            },

            OIHelperCommands::Checkver { path } => {
                let mut cfg_path = path.clone();
                cfg_path.push("oi_ws.json");
                Workspace::from_file(&cfg_path.as_path(), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?.check_version("./oi_ws.json")?;
            },

            OIHelperCommands::Config { key, value } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.set_config(key, value);
                workspace.save_config(Path::new("./oi_ws.json"))?;
            },

            OIHelperCommands::Create { name , template , maxn , maxl, debug_kit } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.check_version("./oi_ws.json")?;
                workspace.create_cpp(name, template, maxn, maxl, *debug_kit)?;
            },

            OIHelperCommands::Run { name, debug } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.check_version("./oi_ws.json")?;
                workspace.run_cpp(name, *debug)?;
            },

            OIHelperCommands::Info => {
                let workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.display_info();
            },

            OIHelperCommands::Update => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.update();
                workspace.save_config(&Path::new("./oi_ws.json"))?;
            },

            OIHelperCommands::GlobalConfig { key, value } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.set_g_config(key, value)?;
            },

            OIHelperCommands::Samples { subcommand } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.check_version("./oi_ws.json")?;
                samples_cli::samples(&mut workspace, subcommand)?;
            },

            OIHelperCommands::Test { target, samples_pack } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"), &self.global_config_path.clone(), &self.global_strings.as_ref().unwrap())?;
                workspace.check_version("./oi_ws.json")?;
                let path_to_sampledir_str;
                if let Some(pack) = samples_pack {
                    path_to_sampledir_str = format!("./{}.smpd", pack.to_owned());
                } else {
                    path_to_sampledir_str = format!("./{}.smpd", target.to_owned());
                }
                let path_to_sampledir = Path::new(&path_to_sampledir_str);
                let mut samples = Samples::from_file(path_to_sampledir.join("samples_info.json").to_str().unwrap())?;
                workspace.test(target, &mut samples)?;
            }
        }
        Ok(())
    }
}
