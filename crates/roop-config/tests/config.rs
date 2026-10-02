use roop_config::{ConfigError, load_config, parse_config};

#[test]
fn defaults_enable_every_target_and_auto_selection() {
    let config = parse_config("Roop.toml", "").unwrap();
    assert!(config.parallel.auto);
    assert_eq!(config.parallel.targets, ["cpu", "metal", "nvptx"]);
    assert_eq!(config.parallel.cpu, None);
}

#[test]
fn reads_modules_and_the_parallel_table() {
    let text = r#"
        [modules]
        std = "std"

        [parallel]
        auto = false
        targets = ["cpu", "metal"]
        cpu = "apple-m4"
    "#;
    let config = parse_config("Roop.toml", text).unwrap();
    assert_eq!(config.modules["std"], "std");
    assert!(!config.parallel.auto);
    assert_eq!(config.parallel.targets, ["cpu", "metal"]);
    assert_eq!(config.parallel.cpu.as_deref(), Some("apple-m4"));
}

#[test]
fn rejects_unknown_targets_and_unknown_keys() {
    let bad_target = parse_config("Roop.toml", "[parallel]\ntargets = [\"tpu\"]");
    assert!(matches!(bad_target, Err(ConfigError::UnknownTarget(t)) if t == "tpu"));
    assert!(parse_config("Roop.toml", "[parallel]\nturbo = true").is_err());
}

#[test]
fn the_repository_config_loads() {
    let root = format!("{}/../../roop/examples", env!("CARGO_MANIFEST_DIR"));
    let config = load_config(std::path::Path::new(&root)).unwrap();
    assert_eq!(config.modules["std"], "std");
}
