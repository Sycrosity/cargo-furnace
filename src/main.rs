use std::path::PathBuf;

use cargo_manifest::Manifest;
use clap::{Args, Parser};

use anyhow::Result;

// use serde::{Deserialize, Serialize};
use tauri_bundler::{BundleBinary, PackageSettings, SettingsBuilder};

use anyhow::*;

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
    pub bin: Option<String>,
    ///Build only the specified example
    #[arg(long, value_name = "NAME")]
    pub example: Option<String>,

    ///Build artifacts in release mode, with optimizations
    #[arg(short, long, group = "cargo-profile")]
    pub release: bool,

    ///Build artifacts with the specified profile
    #[arg(long, group = "cargo-profile", value_name = "PROFILE-NAME")]
    pub profile: Option<String>,

    ///Require Cargo.lock is up to date
    #[arg(long)]
    pub locked: bool,

    ///Space or comma separated list of features to activate
    #[arg(short = 'F', long, value_name = "FEATURES")]
    pub features: Option<Vec<String>>,

    ///Activate all available features
    #[arg(long)]
    pub all_features: bool,

    ///Build for the target triple
    #[arg(long, value_name = "TRIPLE")]
    pub target: Option<String>,

    ///Directory for all generated artifacts - the bundled artifact will be in the directory (target_dir)/furnace/(the artifact name)
    #[arg(long, value_name = "DIRECTORY", default_value = "target/")]
    pub target_dir: PathBuf,

    ///Path to Cargo.toml
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,

    ///Path to Furnace.toml
    #[arg(long, value_name = "PATH", default_value = "Furnace.toml")]
    pub config_path: Option<PathBuf>,

    ///Whether to build the binary or not. ~~Default true~~
    #[arg(long)]
    pub build: bool,

    ///Do not activate the `default` feature
    #[arg(long)]
    pub no_default_features: bool,

    //Whether this is a workspace or not. Default false.
    #[arg(long)]
    pub workspace: bool,
    // // #[arg(long, short = 'o', default_value = "target/cooked")]
    // // pub out_dir: Option<PathBuf>,
    // /// Space or comma separated list of bundles to package.
    // ///
    // /// Each bundle must be one of `deb`, `appimage`, `msi`, `app` or `dmg` on MacOS and `updater` on all platforms.
    // /// If `none` is specified, the bundler will be skipped.
    // ///
    // /// Note that the `updater` bundle is not automatically added so you must specify it if the updater is enabled.
    // #[clap(short, long, action = clap::ArgAction::Append, num_args(0..))]
    // pub bundles: Option<Vec<String>>,
}

fn build_project_if_unbuilt(furnace: &Furnace) -> Result<()> {
    if !furnace.build {
        return Ok(());
    }

    let mut args = vec!["build".to_string()];
    if let Some(triple) = &furnace.target {
        args.push(format!("--target={triple}"));
    }
    if let Some(features) = &furnace.features {
        args.push(format!(
            "--features={}",
            features
                .iter()
                .map(|x| x.to_string() + " ")
                .collect::<String>()
        ));
    }

    args.push(format!("--target-dir={}", furnace.target_dir.display()));

    if let Some(bin) = &furnace.bin {
        args.push(format!("--bin={bin}"));
    }

    let profile = if furnace.release {
        "release"
    } else if let Some(profile) = &furnace.profile {
        if profile == "debug" {
            bail!("Profile name `debug` is reserved")
        }
        profile.as_str()
    } else {
        "dev"
    };

    match profile {
        "dev" => {}
        "release" => {
            args.push("--release".to_string());
        }
        custom => {
            args.push("--profile".to_string());
            args.push(custom.to_string());
        }
    }

    if furnace.all_features {
        args.push("--all-features".to_string());
    }
    if furnace.no_default_features {
        args.push("--no-default-features".to_string());
    }
    let status = std::process::Command::new("cargo").args(args).status()?;
    if !status.success() {
        bail!(
            "Result of `cargo build` operation was unsuccessful: {}",
            status
        );
    }
    Ok(())
}

// struct FurnaceSettings {

// }

fn main() -> Result<()> {
    let CargoCli::Furnace(args) = CargoCli::parse();

    build_project_if_unbuilt(&args)?;

    println!("{:#?}", args);

    // let furnace_config: FurnaceSettings = toml::from_str(
    //     &std::fs::read_to_string(std::env!("CARGO_MANIFEST_DIR"))? + args.config_path);

    // Read config files hierarchically from the current directory, merge them,
    // apply environment variables, and resolve relative paths.
    let manifest = Manifest::from_path(args.manifest_path.unwrap_or("Cargo.toml".into()))?;

    let manifest_package = manifest
        .package
        .expect("cargo manifest must have a \"[package]\" field!");

    let package = if args.workspace {
        // PackageSettings {
        //     product_name: package.name,
        //     version: package.version,
        //     description: package.description.unwrap_or(MaybeInherited::Local("".into())),
        //     homepage: package.homepage,
        //     authors: package.authors,
        //     default_run: package.default_run,
        // }

        unreachable!()
    } else {
        PackageSettings {
            product_name: manifest_package.name.clone(),
            version: manifest_package.version.as_local().expect(
                "Workspace derived config can only be used when --workspace flag is enabled.",
            ),
            description: manifest_package
                .description
                .map(|x| {
                    x.as_local().expect(
                    "Workspace derived config can only be used when --workspace flag is enabled.",
                )
                })
                .unwrap_or("".into()),
            homepage: manifest_package.homepage.map(|x| {
                x.as_local().expect(
                    "Workspace derived config can only be used when --workspace flag is enabled.",
                )
            }),
            authors: manifest_package.authors.map(|x| {
                x.as_local().expect(
                    "Workspace derived config can only be used when --workspace flag is enabled.",
                )
            }),
            default_run: manifest_package.default_run,
        }
    };

    // let settings = SettingsBuilder::new()
    //     .bundle_settings(BundleSettings {
    //         identifier: furnace_config.identifier,
    //         publisher: furnace_config.publisher,
    //         icon: todo!(),
    //         resources: todo!(),
    //         copyright: todo!(),
    //         category: todo!(),
    //         short_description: todo!(),
    //         long_description: todo!(),
    //         bin: todo!(),
    //         external_bin: todo!(),
    //         deb: todo!(),
    //         macos: todo!(),
    //         updater: todo!(),
    //         windows: todo!(),
    //     })
    //     .package_settings(furnace_config.package)
    //     .build()?;

    let bin_location: PathBuf = match args.target {
        Some(target) => PathBuf::from(target),
        None => {
            // match args.profile {
            //     Some(profile) => {

            //         PathBuf::from(profile);

            //     },
            //     None => todo!(),
            // }

            PathBuf::new()
        }
    }
    .join(match args.profile {
        Some(profile) => PathBuf::from(profile),
        None => PathBuf::from("debug"),
    })
    .join(PathBuf::from(manifest_package.name));

    println!("{}",bin_location.display());

    let settings_builder = SettingsBuilder::new();
    let settings = settings_builder
        .project_out_directory(args.target_dir)
        .binaries(vec![
            BundleBinary::new(bin_location.into_os_string().into_string().expect("OsString can't be converted into a String!"), true).set_src_path(Some("src".to_string()))
        ])
        .package_settings(package)
        .build()?;

    println!("{settings:#?}");

    let bundle_paths = tauri_bundler::bundle_project(settings)?;

    println!("{bundle_paths:#?}");
    Ok(())
}
