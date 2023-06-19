/// The type of the package we're bundling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PackageType {
    /// The macOS application bundle (.app).
    MacOsBundle,
    /// The iOS app bundle.
    IosBundle,
    /// The Windows bundle (.msi).
    WindowsMsi,
    /// The NSIS bundle (.exe).
    Nsis,
    /// The Linux Debian package bundle (.deb).
    Deb,
    /// The Linux RPM bundle (.rpm).
    Rpm,
    /// The Linux AppImage bundle (.AppImage).
    AppImage,
    /// The macOS DMG bundle (.dmg).
    Dmg,
}

pub struct Settings {

    

}