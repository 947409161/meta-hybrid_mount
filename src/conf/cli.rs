use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::defs;

#[derive(Parser, Debug)]
#[command(name = "hybrid-mount", version, about = "Hybrid Mount Metamodule")]
pub struct Cli {
    #[arg(short = 'c', long = "config")]
    pub config: Option<PathBuf>,
    #[arg(short = 'm', long = "moduledir")]
    pub moduledir: Option<PathBuf>,
    #[arg(short = 's', long = "mountsource")]
    pub mountsource: Option<String>,
    #[arg(short = 'p', long = "partitions", value_delimiter = ',')]
    pub partitions: Vec<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    GenConfig {
        #[arg(short = 'o', long = "output", default_value = defs::CONFIG_FILE)]
        output: PathBuf,
    },
    ShowConfig,
    #[command(name = "save-config")]
    SaveConfig {
        #[arg(long)]
        payload: String,
    },
    #[command(name = "save-module-rules")]
    SaveModuleRules {
        #[arg(long)]
        module: String,
        #[arg(long)]
        payload: String,
    },
    Modules,
    Conflicts,
    Diagnostics,
    #[command(name = "hymofs")]
    Hymofs {
        #[command(subcommand)]
        action: HymofsAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum HymofsAction {
    Status,
    Add {
        src: String,
        target: String,
        #[arg(long)]
        is_dir: bool,
    },
    AddMerge {
        src: String,
        target: String,
    },
    Del {
        src: String,
    },
    Hide {
        src: String,
    },
    HideXattr {
        src: String,
    },
    Clear,
    List,
    Debug {
        #[arg(long)]
        enable: bool,
    },
    Stealth {
        #[arg(long)]
        enable: bool,
    },
    Enable {
        #[arg(long)]
        enable: bool,
    },
}
