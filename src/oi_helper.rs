//! The main model for OI Helper.

use std::path::Path;

use crate::OIHelperCommands;

use self::workspace::Workspace;

mod workspace;
mod resource;


/// The main model of the OI Helper.
pub struct OIHelper {
    /// Commandline arguments
    args: crate::OIHelperCli,
}

impl OIHelper {
    pub fn new(args: crate::OIHelperCli) -> Self {
        Self { args }
    }
    
    pub fn run(&mut self) {
        match &self.args.subcommand {

            OIHelperCommands::Init { path } => {
                Workspace::create(&path);
            },

            OIHelperCommands::Checkver { path } => {
                let mut cfg_path = path.clone();
                cfg_path.push("oi_ws.json");
                Workspace::from_file(&cfg_path.as_path()).check_version("./oi_ws.json");
            },

            OIHelperCommands::Config { key, value } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"));
                workspace.set_config(key, value);
                workspace.save_config(Path::new("./oi_ws.json"));
            },

            OIHelperCommands::Create { name , template} => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"));
                workspace.check_version("./oi_ws.json");
                workspace.create_cpp(name, template);
            },

            OIHelperCommands::Run { name } => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"));
                workspace.check_version("./oi_ws.json");
                workspace.run_cpp(name);
            },

            OIHelperCommands::Info => {
                let workspace = Workspace::from_file(Path::new("./oi_ws.json"));
                workspace.display_info();
            },

            OIHelperCommands::Update => {
                let mut workspace = Workspace::from_file(Path::new("./oi_ws.json"));
                workspace.update();
                workspace.save_config(&Path::new("./oi_ws.json"));
            }

        }
    }
}
