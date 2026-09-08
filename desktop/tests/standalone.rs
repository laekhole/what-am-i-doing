use std::{
    fs,
    io::Read,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};

struct Fixture {
    dir: PathBuf,
    child: Option<Child>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn renamed_executable_collects_and_contains_license_without_companion_files() {
    let mut fixture = Fixture {
        dir: std::env::temp_dir().join(format!(
            "waid-standalone-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )),
        child: None,
    };
    let download = fixture.dir.join("다운로드 폴더");
    let logs = fixture.dir.join("sessions");
    fs::create_dir_all(&download).unwrap();
    fs::create_dir_all(&logs).unwrap();
    fs::write(
        logs.join("session.jsonl"),
        "{\"type\":\"user\",\"message\":{\"content\":\"single executable fixture\"}}\n",
    )
    .unwrap();
    let exe = download.join(if cfg!(windows) {
        "renamed app.exe"
    } else {
        "renamed app"
    });
    fs::copy(env!("CARGO_BIN_EXE_waid-desktop"), &exe).unwrap();
    for (args, license) in [
        (
            vec!["--waid-core", "--json", "--history", "--agent", "codex"],
            false,
        ),
        (vec!["--licenses"], true),
    ] {
        let mut command = Command::new(&exe);
        command
            .args(args)
            .current_dir(&download)
            .env("HOME", &fixture.dir)
            .env("USERPROFILE", &fixture.dir)
            .env("CODEX_HOME", &fixture.dir)
            .env("WAID_ADAPTERS", fixture.dir.join("adapters"))
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        fixture.child = Some(command.spawn().unwrap());
        let mut stdout = fixture.child.as_mut().unwrap().stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut text = String::new();
            let result = stdout.read_to_string(&mut text).map(|_| text);
            let _ = tx.send(result);
        });
        let output = rx
            .recv_timeout(Duration::from_secs(15))
            .expect("standalone mode must exit")
            .unwrap();
        assert!(fixture.child.as_mut().unwrap().wait().unwrap().success());
        fixture.child = None;
        if license {
            assert_eq!(output, include_str!("../assets/fonts/LICENSE.txt"));
        } else {
            let snapshot: serde_json::Value = serde_json::from_str(&output).unwrap();
            assert_eq!(snapshot["schema"], 2);
            assert!(snapshot["sessions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["prompt"] == "single executable fixture"));
        }
        assert_eq!(
            fs::read_dir(&download).unwrap().count(),
            1,
            "no extracted executables or companion files"
        );
    }
}
