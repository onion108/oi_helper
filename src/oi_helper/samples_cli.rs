use std::{path::{Path, self}, process::exit, fs::{File, self}};

use crossterm::style::Stylize;

use crate::SamplesSubcommand;

use super::{workspace::Workspace, samples::Samples};



pub fn samples(workspace: &mut Workspace, subcommand: &SamplesSubcommand) {
    
    match subcommand {
        SamplesSubcommand::Init { name } => {

            let path_to_sampledir_str = name.to_owned() + ".smpd";
            let path_to_sampledir = Path::new(&path_to_sampledir_str);
            if path_to_sampledir.exists() && !path_to_sampledir.is_dir() {
                eprintln!("{}", format!("Cannot create the sample because the filename {} has been used. Please check your directory.", path_to_sampledir_str).bold().red());
                exit(-1);
            }
            if !path_to_sampledir.exists() {
                match fs::create_dir(path_to_sampledir) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("{}", format!("Cannot create the sample because the "));
                        exit(-1);
                    }
                }
            }
            let config_path = path_to_sampledir.join("samples_info.json");
            Samples::create(config_path.to_str().unwrap());

        }
        SamplesSubcommand::Create { name, timeout, memory_limit, points } => {
            //
        }
    }

}
