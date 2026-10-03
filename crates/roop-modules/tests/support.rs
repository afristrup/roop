#![allow(dead_code)]

use roop_config::{Config, load_config};
use roop_modules::{ModuleError, load_program};
use roop_syntax::{Item, Program};
use std::path::{Path, PathBuf};

/// Writes `files` (path, text) into a fresh directory and returns it.
pub fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("roop-mod-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let file = dir.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }
    dir
}

pub fn load(dir: &Path) -> Result<Program, ModuleError> {
    let config: Config = load_config(dir).unwrap();
    load_program(&dir.join("prog.roop"), &config)
}

pub fn fn_names(program: &Program) -> Vec<String> {
    program
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect()
}
