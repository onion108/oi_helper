//! The entry file of the CLI.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use crossterm::style::Stylize;

mod oi_helper;

/// The version of the command-line tool.
pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static DEBUG: bool = false;

pub fn is_debug() -> bool {
    match std::env::var("APP_DEBUG") {
        Ok(s) => s.trim().to_uppercase() == "YES",
        Err(_) => false,
    }
}

pub struct MemoryLeakTracer;

impl Drop for MemoryLeakTracer {
    fn drop(&mut self) {
        println!("No memory leak. ");
    }
}

/// Subcommands for the sample.
#[derive(Subcommand)]
pub enum SamplesSubcommand {

    /// Initialize a new samples group for a source file.
    Init {

        /// The name of the source file without the extension.
        #[clap()]
        name: String,

    },

    /// Create a sample,
    Create {

        /// The name of the source file without the extension.
        #[clap()]
        name: String,

        /// The timeout of the sample.
        #[clap(long, value_parser, default_value_t = 1000)]
        timeout: u32,

        /// The memory limit.
        #[clap(long, value_parser, default_value_t = 256)]
        memory_limit: u32,
        
        /// The points
        #[clap(long, value_parser, default_value_t = 10)]
        points: u32,

    },

    /// Fetch example I/O groups from Luogu.
    Lgfetch {

        /// The name of fetched case.
        #[clap()]
        name: String,

        /// The problem id
        #[clap()]
        problem_id: String,

    },

}

/// Subcommands
#[derive(Subcommand)]
enum OIHelperCommands {

    /// Check the version
    Checkver {
        /// The path to the workspace to check.
        #[clap(parse(from_os_str), default_value=".")]
        path: std::path::PathBuf,
    },

    /// Initialize a workspace.
    Init {
        /// The path to the workspace directory
        #[clap(parse(from_os_str), default_value=".")]
        path: std::path::PathBuf,
    },

    /// Config current workspace.
    Config {
        /// The key of an option.  
        /// E.g. `cc_flags`.
        #[clap()]
        key: String,

        /// The value of an option.
        /// E.g. `-std=c++17 -xc++ -O1 -Wall`
        #[clap()]
        value: String,
    },

    /// Edit the global configuration, which will be used when initializing workspace or updating oi_ws.json.
    GlobalConfig {
        /// The key of an option.
        /// E.g. `cc_flags`
        #[clap()]
        key: String,

        /// The value of an option.
        /// E.g. `-std=c++17 -xc++ -O1 -Wall`
        #[clap()]
        value: String,
    },

    /// Create a C++ source file in the workspace.
    Create {
        /// The name of the source file, the extension isn't neccessary.
        #[clap()]
        name: String,

        /// The template name. Defaults to default.
        #[clap(short='t', long, default_value="default")]
        template: String,

        /// The value of the `MAXN` constant.
        #[clap(long, default_value="1e5+114514")]
        maxn: String,

        /// The value of the `MAXL` constant, will be used in the 2d dp template.
        #[clap(long, default_value="128")]
        maxl: String,

        /// Determine if enable the debug kit.
        #[clap(short='d', long)]
        debug_kit: bool,
    },

    /// Run the program.
    Run {
        /// The name of the source file, the extension isn't neccessary.
        #[clap()]
        name: String,

        /// Determine if debug kit is enabled (so the things in debug({}) will be executed. By default, they won't be executed unless you enable this option.)
        #[clap(short='d', long)]
        debug: bool,
    },

    /// Display the info of current workspace.
    Info,

    /// Update the workspace to the newest oi_helper version.
    Update,

    /// Edit the sample group.
    Samples {
        #[clap(subcommand)]
        subcommand: SamplesSubcommand,
    },

    /// Run the sample group on a target.
    Test {
        /// The file name without extension.
        #[clap()]
        target: String,

        /// The path-to-samples-directory (without .smpd extension). If not specified, it will be the same as the target.
        #[clap(short='s', long)]
        samples_pack: Option<String>, 
    },

}

/// A helper for C++ competive programmers (a.k.a. OIers).
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct OIHelperCli {

    /// The subcommand.
    #[clap(subcommand)]
    subcommand: OIHelperCommands,

}

#[allow(unused_assignments)]
fn main() -> ExitCode {
    let _watcher;
    if DEBUG {
        _watcher = MemoryLeakTracer;
    }
    let args = OIHelperCli::parse();
    let mut app = oi_helper::OIHelper::new(args);
    app.config();
    match app.run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(msg) => {
            if let Some(msg) = msg {
                println!("{}", format!("Error: {}", msg).bold().red());
            }
            ExitCode::FAILURE
        }
    }
}
