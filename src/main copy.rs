use std::path::PathBuf;

use clap::{Args, Parser};

use serde::{Deserialize, Serialize};
use tauri_bundler::{
    bundle::{Settings, SettingsBuilder},
    DebianSettings, BundleSettings,
};

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

    ///Directory for all generated artifacts
    #[arg(long, value_name = "DIRECTORY")]
    pub target_dir: Option<PathBuf>,

    ///Path to Cargo.toml
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,

    ///Path to Furnace.toml
    #[arg(long, value_name = "PATH", default_value = "Furnace.toml")]
    pub config_path: Option<PathBuf>,

    ///whether to build the binary or not. Default true
    #[arg(long)]
    pub build: bool,

    ///Do not activate the `default` feature
    #[arg(long)]
    pub no_default_features: bool,

    /// Space or comma separated list of bundles to package.
    ///
    /// Each bundle must be one of `deb`, `appimage`, `msi`, `app` or `dmg` on MacOS and `updater` on all platforms.
    /// If `none` is specified, the bundler will be skipped.
    ///
    /// Note that the `updater` bundle is not automatically added so you must specify it if the updater is enabled.
    #[clap(short, long, action = clap::ArgAction::Append, num_args(0..))]
    pub bundles: Option<Vec<String>>,
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

#[derive(Serialize, Deserialize, Debug)]
struct FurnaceSettings {
    pub package: FurnacePackageSettings,

    pub identifier: Option<String>,

    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub icon: Option<Vec<PathBuf>>,
    pub resources: Option<Vec<PathBuf>>,
    pub copyright: Option<String>,
    pub category: Option<String>,
    pub deb: Option<FurnaceDebianSettings>,
    pub macos: Option<FurnaceMacosSettings>,
    // external_bin:
}

/// The bundle settings of the BuildArtifact we're bundling.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FurnaceBundleSettings {
  /// the app's identifier.
  pub identifier: Option<String>,
  /// The app's publisher. Defaults to the second element in the identifier string.
  /// Currently maps to the Manufacturer property of the Windows Installer.
  pub publisher: Option<String>,
  /// the app's icon list.
  pub icon: Option<Vec<String>>,
  /// the app's resources to bundle.
  ///
  /// each item can be a path to a file or a path to a folder.
  ///
  /// supports glob patterns.
  pub resources: Option<Vec<String>>,
  /// the app's copyright.
  pub copyright: Option<String>,
  /// the app's category.
  pub category: Option<AppCategory>,
  /// the app's short description.
  pub short_description: Option<String>,
  /// the app's long description.
  pub long_description: Option<String>,
  // Bundles for other binaries:
  /// Configuration map for the apps to bundle.
  pub bin: Option<HashMap<String, BundleSettings>>,
  /// External binaries to add to the bundle.
  ///
  /// Note that each binary name should have the target platform's target triple appended,
  /// as well as `.exe` for Windows.
  /// For example, if you're bundling a sidecar called `sqlite3`, the bundler expects
  /// a binary named `sqlite3-x86_64-unknown-linux-gnu` on linux,
  /// and `sqlite3-x86_64-pc-windows-gnu.exe` on windows.
  ///
  /// Run `tauri build --help` for more info on targets.
  ///
  /// If you are building a universal binary for MacOS, the bundler expects
  /// your external binary to also be universal, and named after the target triple,
  /// e.g. `sqlite3-universal-apple-darwin`. See
  /// <https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary>
  pub external_bin: Option<Vec<String>>,
  /// Debian-specific settings.
  pub deb: tauri_bundler::DebianSettings,
  /// MacOS-specific settings.
  pub macos: tauri_bundler::MacOsSettings,
  /// Windows-specific settings.
  pub windows: tauri_bundler::WindowsSettings,
}



#[derive(Serialize, Deserialize, Debug)]
pub struct FurnacePackageSettings {
    /// the package's product name.
    pub name: Option<String>,
    /// the package's version.
    pub version: Option<String>,
    /// the package's description.
    pub description: Option<String>,
    /// the package's homepage.
    pub homepage: Option<String>,
    /// the package's authors.
    pub authors: Option<Vec<String>>,
    /// the default binary to run.
    pub default_run: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FurnaceDebianSettings {
    // OS-specific settings:
    /// the list of debian dependencies.
    pub depends: Option<Vec<String>>,
    //   /// List of custom files to add to the deb package.
    //   /// Maps the path on the debian package to the path of the file to include (relative to the current working directory).
    //   pub files: HashMap<PathBuf, PathBuf>,
    /// Path to a custom desktop file Handlebars template.
    ///
    /// Available variables: `categories`, `comment` (optional), `exec`, `icon` and `name`.
    ///
    /// Default file contents:
    /// ```text
    #[doc = "[Desktop Entry]
Categories={{categories}}
{{#if comment}}
Comment={{comment}}
   {{/if}}
Exec={{exec}}
Icon={{icon}}
Name={{name}}
Terminal=false
Type=Application"]
    /// ```
    pub desktop_template: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FurnaceMacosSettings {
    /// MacOS frameworks that need to be bundled with the app.
    ///
    /// Each string can either be the name of a framework (without the `.framework` extension, e.g. `"SDL2"`),
    /// in which case we will search for that framework in the standard install locations (`~/Library/Frameworks/`, `/Library/Frameworks/`, and `/Network/Library/Frameworks/`),
    /// or a path to a specific framework bundle (e.g. `./data/frameworks/SDL2.framework`).  Note that this setting just makes tauri-bundler copy the specified frameworks into the OS X app bundle
    /// (under `Foobar.app/Contents/Frameworks/`); you are still responsible for:
    ///
    /// - arranging for the compiled binary to link against those frameworks (e.g. by emitting lines like `cargo:rustc-link-lib=framework=SDL2` from your `build.rs` script)
    ///
    /// - embedding the correct rpath in your binary (e.g. by running `install_name_tool -add_rpath "@executable_path/../Frameworks" path/to/binary` after compiling)
    pub frameworks: Option<Vec<String>>,
    /// A version string indicating the minimum MacOS version that the bundled app supports (e.g. `"10.11"`).
    /// If you are using this config field, you may also want have your `build.rs` script emit `cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.11`.
    pub minimum_system_version: Option<String>,
    /// The path to the LICENSE file for macOS apps.
    /// Currently only used by the dmg bundle.
    pub license: Option<String>,
    /// The exception domain to use on the macOS .app bundle.
    ///
    /// This allows communication to the outside world e.g. a web server you're shipping.
    pub exception_domain: Option<String>,
    //   /// Code signing identity.
    //   pub signing_identity: Option<String>,
    //   /// Provider short name for notarization.
    //   pub provider_short_name: Option<String>,
    //   /// Path to the entitlements.plist file.
    //   pub entitlements: Option<String>,
    //   /// Path to the Info.plist file for the bundle.
    //   pub info_plist_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let CargoCli::Furnace(args) = CargoCli::parse();

    build_project_if_unbuilt(&args);

    println!("{:#?}", args);

    let cwd = std::env::current_dir().unwrap();

    let furnace_config: FurnaceSettings = toml::from_str(
        &std::fs::read_to_string(std::env!("CARGO_MANIFEST_DIR"))? + args.config_path,
    )?;

    let settings = SettingsBuilder::new()
        .bundle_settings(BundleSettings {
            identifier: furnace_config.identifier,
            publisher: furnace_config.publisher,
            icon: todo!(),
            resources: todo!(),
            copyright: todo!(),
            category: todo!(),
            short_description: todo!(),
            long_description: todo!(),
            bin: todo!(),
            external_bin: todo!(),
            deb: todo!(),
            macos: todo!(),
            updater: todo!(),
            windows: todo!(),
        })
        .package_settings(furnace_config.package)
        .build()?;

    let bundle_paths = tauri_bundler::bundle_project(settings)?;

    println!("{bundle_paths:#?}");
    Ok(())
}
