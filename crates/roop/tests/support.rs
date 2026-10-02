#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ROOP: &str = env!("CARGO_BIN_EXE_roop");

pub fn project(name: &str, config: &str, source: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("roop-cli-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Roop.toml"), config).unwrap();
    std::fs::write(dir.join("prog.roop"), source).unwrap();
    dir
}

pub fn roop(dir: &Path, args: &[&str]) -> Output {
    Command::new(ROOP)
        .current_dir(dir)
        .args(args)
        .env("ROOP_RT_LIB", runtime_lib())
        .output()
        .unwrap()
}

pub fn runtime_lib() -> PathBuf {
    static BUILD: std::sync::Once = std::sync::Once::new();
    BUILD.call_once(|| {
        let cargo = Command::new(env!("CARGO"));
        let status = {
            let mut c = cargo;
            c.args(["build", "-p", "roop-rt"]).status().unwrap()
        };
        assert!(status.success());
    });
    let target = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| format!("{}/../../target", env!("CARGO_MANIFEST_DIR")));
    PathBuf::from(target).join("debug/libroop_rt.a")
}

pub fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}
