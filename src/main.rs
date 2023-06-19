mod bundle;
mod settings;

use std::path::PathBuf;

use clap::{Args, Parser};

use anyhow::Result;

#[derive(Parser, Debug)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    Furnace(Furnace),
}

#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
struct Furnace {
    ///Build only the specified binary
    #[arg(long, value_name = "NAME")]
    bin: Option<String>,
    ///Build only the specified example
    #[arg(long, value_name = "NAME")]
    example: Option<String>,

    ///Build artifacts in release mode, with optimizations
    #[arg(short, long, group = "cargo-profile")]
    release: bool,

    ///Build artifacts with the specified profile
    #[arg(long, group = "cargo-profile", value_name = "PROFILE-NAME")]
    profile: Option<String>,

    ///Require Cargo.lock is up to date
    #[arg(long)]
    locked: bool,

    ///Space or comma separated list of features to activate
    #[arg(short = 'F', long, value_name = "FEATURES")]
    features: Vec<String>,

    ///Activate all available features
    #[arg(long)]
    all_features: bool,

    ///Build for the target triple
    #[arg(long, value_name = "TRIPLE")]
    target: Option<String>,

    ///Directory for all generated artifacts
    #[arg(long, value_name = "DIRECTORY")]
    target_dir: Option<PathBuf>,

    ///Path to Cargo.toml
    #[arg(long, value_name = "PATH")]
    manifest_path: Option<PathBuf>,

    ///Path to Furnace.toml
    #[arg(long, value_name = "PATH", default_value = "Furnace.toml")]
    config_path: Option<PathBuf>,
}

fn main() {
    let CargoCli::Furnace(args) = CargoCli::parse();

    println!("{:#?}", args);
}
