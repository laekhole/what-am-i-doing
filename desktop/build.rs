use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "macos" {
        build_macos();
    }
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        embed_icon();
    }
}

fn build_macos() {
    println!("cargo:rerun-if-changed=src/macos/App.swift");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let resources = out.join("Resources.swift");
    let mut source = String::new();
    for (name, path) in [("daylight", "templates/daylight.json"), ("midnight", "templates/midnight.json"), ("fontLicense", "assets/fonts/LICENSE.txt")] {
        println!("cargo:rerun-if-changed={path}");
        let text = std::fs::read_to_string(path).unwrap();
        source.push_str(&format!("let {name} = ###\"\"\"\n{text}\n\"\"\"###\n"));
    }
    std::fs::write(&resources, source).unwrap();
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "aarch64" => "arm64", "x86_64" => "x86_64", other => panic!("unsupported Mac architecture: {other}"),
    };
    let status = Command::new("xcrun").args([
        "swiftc", "-swift-version", "5", "-O", "-parse-as-library", "-emit-library", "-static",
        "-module-name", "WaidApp", "-target", &format!("{arch}-apple-macosx13.0"),
        "src/macos/App.swift", resources.to_str().unwrap(), "-o",
    ]).arg(out.join("libWaidApp.a")).status().expect("Xcode command line tools (xcrun swiftc)");
    assert!(status.success(), "AppKit shell compilation failed");
    let swift = Command::new("xcrun").args(["--find", "swiftc"]).output().unwrap();
    assert!(swift.status.success());
    let compiler = PathBuf::from(String::from_utf8(swift.stdout).unwrap().trim());
    let libraries = compiler.parent().unwrap().parent().unwrap().join("lib/swift/macosx");
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-search=native={}", libraries.display());
    println!("cargo:rustc-link-lib=static=WaidApp");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
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
