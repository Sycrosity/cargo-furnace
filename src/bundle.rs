/// Bundles the project.
/// Returns the list of paths where the bundles can be found.
pub fn bundle_project(settings: Settings) -> crate::Result<Vec<Bundle>> {
    let package_types = settings.package_types()?;
    if package_types.is_empty() {
        return Ok(Vec::new());
    }

    let mut bundles: Vec<Bundle> = Vec::new();

    let target_os = settings
        .target()
        .split('-')
        .nth(2)
        .unwrap_or(std::env::consts::OS)
        .replace("darwin", "macos");

    if target_os != std::env::consts::OS {
        warn!("Cross-platform compilation is experimental and does not support all features. Please use a matching host system for full compatibility.");
    }

    for package_type in &package_types {
        // bundle was already built! e.g. DMG already built .app
        if bundles.iter().any(|b| b.package_type == *package_type) {
            continue;
        }

        let bundle_paths = match package_type {
            #[cfg(target_os = "macos")]
            PackageType::MacOsBundle => macos::app::bundle_project(&settings)?,
            #[cfg(target_os = "macos")]
            PackageType::IosBundle => macos::ios::bundle_project(&settings)?,
            // dmg is dependant of MacOsBundle, we send our bundles to prevent rebuilding
            #[cfg(target_os = "macos")]
            PackageType::Dmg => {
                let bundled = macos::dmg::bundle_project(&settings, &bundles)?;
                if !bundled.app.is_empty() {
                    bundles.push(Bundle {
                        package_type: PackageType::MacOsBundle,
                        bundle_paths: bundled.app,
                    });
                }
                bundled.dmg
            }

            #[cfg(target_os = "windows")]
            PackageType::WindowsMsi => windows::msi::bundle_project(&settings, false)?,
            PackageType::Nsis => windows::nsis::bundle_project(&settings, false)?,

            #[cfg(target_os = "linux")]
            PackageType::Deb => linux::debian::bundle_project(&settings)?,
            #[cfg(target_os = "linux")]
            PackageType::Rpm => linux::rpm::bundle_project(&settings)?,
            #[cfg(target_os = "linux")]
            PackageType::AppImage => linux::appimage::bundle_project(&settings)?,

            // updater is dependant of multiple bundle, we send our bundles to prevent rebuilding
            PackageType::Updater => {
                if !package_types.iter().any(|p| {
                    matches!(
                        p,
                        PackageType::AppImage
                            | PackageType::MacOsBundle
                            | PackageType::Nsis
                            | PackageType::WindowsMsi
                    )
                }) {
                    warn!("The updater bundle target exists but couldn't find any updater-enabled target, so the updater artifacts won't be generated. Please add one of these targets as well: app, appimage, msi, nsis");
                    continue;
                }
                updater_bundle::bundle_project(&settings, &bundles)?
            }
            _ => {
                warn!("ignoring {}", package_type.short_name());
                continue;
            }
        };

        bundles.push(Bundle {
            package_type: package_type.to_owned(),
            bundle_paths,
        });
    }

    #[cfg(target_os = "macos")]
    {
        // Clean up .app if only building dmg or updater
        if !package_types.contains(&PackageType::MacOsBundle) {
            if let Some(app_bundle_paths) = bundles
                .iter()
                .position(|b| b.package_type == PackageType::MacOsBundle)
                .map(|i| bundles.remove(i))
                .map(|b| b.bundle_paths)
            {
                for app_bundle_path in &app_bundle_paths {
                    info!(action = "Cleaning"; "{}", app_bundle_path.display());
                    match app_bundle_path.is_dir() {
                        true => std::fs::remove_dir_all(app_bundle_path),
                        false => std::fs::remove_file(app_bundle_path),
                    }
                    .with_context(|| {
                        format!(
                            "Failed to clean the app bundle at {}",
                            app_bundle_path.display()
                        )
                    })?
                }
            }
        }
    }

    if !bundles.is_empty() {
        let bundles_wo_updater = bundles
            .iter()
            .filter(|b| b.package_type != PackageType::Updater)
            .collect::<Vec<_>>();
        let pluralised = if bundles_wo_updater.len() == 1 {
            "bundle"
        } else {
            "bundles"
        };

        let mut printable_paths = String::new();
        for bundle in &bundles {
            for path in &bundle.bundle_paths {
                let mut note = "";
                if bundle.package_type == crate::PackageType::Updater {
                    note = " (updater)";
                }
                writeln!(printable_paths, "        {}{}", display_path(path), note).unwrap();
            }
        }

        info!(action = "Finished"; "{} {} at:\n{}", bundles_wo_updater.len(), pluralised, printable_paths);

        Ok(bundles)
    } else {
        Err(anyhow::anyhow!("No bundles were built").into())
    }
}

/// Check to see if there are icons in the settings struct
pub fn check_icons(settings: &Settings) -> crate::Result<bool> {
    // make a peekable iterator of the icon_files
    let mut iter = settings.icon_files().peekable();

    // if iter's first value is a None then there are no Icon files in the settings struct
    if iter.peek().is_none() {
        Ok(false)
    } else {
        Ok(true)
    }
}
