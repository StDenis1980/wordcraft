//! Windows only: embed the app icon and version info (VERSIONINFO) into `wordcraft.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `WORDCRAFT_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/wordcraft.ico");
    println!("cargo:rerun-if-env-changed=WORDCRAFT_REQUIRE_WINRES");
    // The product name and publisher (wordcraft-brand); unset, the release's.
    println!("cargo:rerun-if-env-changed=WORDCRAFT_BRAND_NAME");
    println!("cargo:rerun-if-env-changed=WORDCRAFT_BRAND_PUBLISHER");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let brand = |var: &str| std::env::var(var).ok().filter(|v| !v.is_empty());
    let name = brand("WORDCRAFT_BRAND_NAME");
    // A rebranded build is published by someone else: their name, or none.
    let company = match &name {
        Some(_) => brand("WORDCRAFT_BRAND_PUBLISHER").unwrap_or_default(),
        None => "Learning Machines LLC".to_owned(),
    };
    let name = name.unwrap_or_else(|| "WordCraft".to_owned());
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/wordcraft.ico")
        .set("ProductName", &name)
        .set("FileDescription", &format!("{name} word processor"))
        .set("CompanyName", &company)
        .set("LegalCopyright", "Copyright (c) the WordCraft authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "wordcraft.exe")
        .set("InternalName", "wordcraft");
    if let Err(e) = res.compile() {
        if std::env::var_os("WORDCRAFT_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=wordcraft.exe built without icon/version resources: {e}");
    }
}
