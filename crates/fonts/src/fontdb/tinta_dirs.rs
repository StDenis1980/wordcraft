//! Tinta Office: font folders beyond the system's, scanned like them ([`super::system_font_dirs`]).
//!
//! - **The fonts that ship with the dictionaries** (`packaging/dictionaries/sources.tsv`: Noto Sans,
//!   OFL), in the dictionaries folder's `fonts` subfolder: next to the executable on Windows,
//!   `share/wordcraft/dictionaries` on Linux and in the Flatpak, `Contents/Resources/dictionaries`
//!   on macOS. Spanish administrations set their documents in Noto Sans, which few computers have;
//!   another font in its place makes every line shorter and narrower, so pages end early.
//! - **Microsoft Office's cloud fonts** (Aptos and the other fonts Office downloads on demand):
//!   Office keeps them in its own cache without installing them in the system, so a document set
//!   in Aptos found no Aptos on a computer that has it. They are read where they are, never copied.

use std::path::{Path, PathBuf};

/// The `fonts` folders of the dictionaries shipped beside `exe`.
pub fn bundled_font_dirs(exe: &Path) -> Vec<PathBuf> {
    let Some(bin) = exe.parent() else { return Vec::new() };
    let mut dirs = vec![bin.join("dictionaries").join("fonts")];
    if let Some(prefix) = bin.parent() {
        dirs.push(prefix.join("share").join("wordcraft").join("dictionaries").join("fonts"));
        dirs.push(prefix.join("Resources").join("dictionaries").join("fonts"));
    }
    dirs
}

/// Office's cloud font caches: under `%LOCALAPPDATA%` on Windows (`local_app_data`), in Office's
/// group container under the home folder on macOS (`home`).
pub fn office_cloud_font_dirs(local_app_data: Option<&Path>, home: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if cfg!(windows)
        && let Some(l) = local_app_data
    {
        dirs.push(l.join("Microsoft").join("FontCache").join("4").join("CloudFonts"));
    }
    if cfg!(target_os = "macos")
        && let Some(h) = home
    {
        dirs.push(h.join("Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts"));
    }
    dirs
}

/// Both, for the running executable and user.
pub(super) fn dirs() -> Vec<PathBuf> {
    let mut dirs = std::env::current_exe().map(|exe| bundled_font_dirs(&exe)).unwrap_or_default();
    let local = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty()).map(PathBuf::from);
    let home = std::env::var_os("HOME").filter(|v| !v.is_empty()).map(PathBuf::from);
    dirs.extend(office_cloud_font_dirs(local.as_deref(), home.as_deref()));
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dictionaries_fonts_folder_is_found_on_every_layout() {
        let exe = Path::new("/opt/tinta/bin/wordcraft");
        assert_eq!(
            bundled_font_dirs(exe),
            [
                PathBuf::from("/opt/tinta/bin/dictionaries/fonts"),
                PathBuf::from("/opt/tinta/share/wordcraft/dictionaries/fonts"),
                PathBuf::from("/opt/tinta/Resources/dictionaries/fonts"),
            ]
        );
        assert!(bundled_font_dirs(Path::new("")).is_empty());
        // The system scan includes them.
        let exe = std::env::current_exe().unwrap();
        let sys = crate::system_font_dirs();
        for d in bundled_font_dirs(&exe) {
            assert!(sys.contains(&d), "{}", d.display());
        }
    }

    #[test]
    fn office_cloud_fonts_are_looked_for_on_windows_and_macos() {
        let dirs = office_cloud_font_dirs(Some(Path::new("/l")), Some(Path::new("/h")));
        let want: Vec<PathBuf> = if cfg!(windows) {
            vec![Path::new("/l").join("Microsoft").join("FontCache").join("4").join("CloudFonts")]
        } else if cfg!(target_os = "macos") {
            vec![PathBuf::from("/h/Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts")]
        } else {
            Vec::new()
        };
        assert_eq!(dirs, want);
        assert!(office_cloud_font_dirs(None, None).is_empty());
    }

    /// A font file in the dictionaries' `fonts` folder is cataloged like an installed one.
    #[test]
    fn a_font_file_in_the_folder_is_cataloged() {
        let dir = std::env::temp_dir().join(format!("wordcraft-tinta-fonts-{}", std::process::id()));
        let fonts = dir.join("dictionaries").join("fonts");
        std::fs::create_dir_all(&fonts).unwrap();
        let Some(data) = super::super::bundled().first() else { return };
        std::fs::write(fonts.join("x.ttf"), data).unwrap();
        let db = crate::FontDb::with_font_dirs(bundled_font_dirs(&dir.join("wordcraft")));
        assert!(db.load_system_fonts() > 0, "the folder is scanned");
        let elsewhere = crate::FontDb::with_font_dirs(bundled_font_dirs(&dir.join("bin").join("wordcraft")));
        assert_eq!(elsewhere.load_system_fonts(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
