fn main() {
    let windows = std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows";
    let suffix = if windows { ".exe" } else { "" };
    println!("cargo:rerun-if-env-changed=WAID_CORE_PATH");
    let core = std::env::var_os("WAID_CORE_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join(format!("../target/release/waid{suffix}"))
        });
    println!("cargo:rerun-if-changed={}", core.display());
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let profile = out.ancestors().nth(3).unwrap();
    let bytes = std::fs::read(&core).expect(
        "build the native waid core first, or set WAID_CORE_PATH to the matching target binary",
    );
    let destination = profile.join(format!("waid{suffix}"));
    // Windows locks running executables. An identical bundled core needs no rewrite.
    if std::fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
        std::fs::write(&destination, bytes)
            .expect("close the app before replacing its bundled core");
    }
}
