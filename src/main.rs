//! The entry file of the CLI.

use clap::{Parser, Subcommand};

mod oi_helper;

/// The version of the command-line tool.
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

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

    /// Create a C++ source file in the workspace.
    Create {
        /// The name of the source file, the extension isn't neccessary.
        #[clap()]
        name: String,

        /// The template name. Defaults to default.
        #[clap(long, default_value="default")]
        template: String,
    },

    /// Run the program.
    Run {
        /// The name of the source file, the extension isn't neccessary.
        #[clap()]
        name: String,
    },

    /// Display the info of current workspace.
    Info,
}

/// A helper for C++ competive programmers (a.k.a. OIers).
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct OIHelperCli {

    /// The subcommand.
    #[clap(subcommand)]
    subcommand: OIHelperCommands,

}

fn main() {
    let args = OIHelperCli::parse();
    oi_helper::OIHelper::new(args).run();
}
