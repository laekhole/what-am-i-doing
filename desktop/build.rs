use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let windows = std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows";
    let suffix = if windows { ".exe" } else { "" };
    if windows {
        embed_icon();
    }
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
    println!("cargo:rerun-if-changed=assets/fonts/LICENSE.txt");
    std::fs::write(
        profile.join("FONT-LICENSE.txt"),
        include_bytes!("assets/fonts/LICENSE.txt"),
    )
    .expect("write the bundled font license");
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

fn embed_icon() {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let resource = manifest_dir.join("icon.rc");
    let icon = manifest_dir.parent().unwrap().join("assets/waid.ico");
    println!("cargo:rerun-if-changed={}", resource.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("app.manifest").display()
    );
    println!("cargo:rerun-if-changed={}", icon.display());

    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let (compiler, output, args): (PathBuf, PathBuf, Vec<String>) = if target_env == "msvc" {
        let compiler = find_tool("rc.exe", &manifest_dir);
        let output = out.join("waid-icon.res");
        let args = vec![
            "/nologo".into(),
            format!("/fo{}", output.display()),
            resource.display().to_string(),
        ];
        (compiler, output, args)
    } else {
        let compiler = find_tool("windres.exe", &manifest_dir);
        let output = out.join("waid-icon.o");
        let mut args = vec![
            "--output-format=coff".into(),
            resource.display().to_string(),
            output.display().to_string(),
        ];
        // ponytail: type handles this literal .rc; use gcc if resource macros are added.
        if !tool_on_path("gcc") {
            args.insert(0, "--preprocessor=type".into());
        }
        (compiler, output, args)
    };

    let status = Command::new(&compiler)
        .current_dir(&manifest_dir)
        .args(args)
        .status()
        .unwrap_or_else(|error| panic!("run {}: {error}", compiler.display()));
    assert!(
        status.success(),
        "resource compiler failed for {}",
        icon.display()
    );
    assert!(
        output.is_file(),
        "resource compiler did not create {}",
        output.display()
    );
    println!("cargo:rustc-link-arg={}", output.display());
}

fn find_tool(name: &str, manifest_dir: &Path) -> PathBuf {
    if name == "windres.exe" {
        let bundled = manifest_dir
            .parent()
            .unwrap()
            .join(".tools/binutils/mingw64/bin")
            .join(name);
        if bundled.is_file() {
            return bundled;
        }
    }
    PathBuf::from(name)
}

fn tool_on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path)
        .any(|dir| dir.join(name).is_file() || dir.join(format!("{name}.exe")).is_file())
}
