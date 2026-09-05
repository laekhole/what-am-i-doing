fn main() {
    let windows = std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows";
    let suffix = if windows { ".exe" } else { "" };
    let core = format!("../target/release/waid{suffix}");
    println!("cargo:rerun-if-changed={core}");
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let profile = out.ancestors().nth(3).unwrap();
    std::fs::copy(&core, profile.join(format!("waid{suffix}")))
        .expect("build the native waid core with cargo build --release first");
}
