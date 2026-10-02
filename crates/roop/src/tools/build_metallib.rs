use crate::{CliError, run_tool};
use std::path::Path;
use std::process::Command;

/// AIR text to a `.metallib`, using Apple's `metal` and `metallib`.
pub fn build_metallib(air_ir: &str, dir: &Path) -> Result<Vec<u8>, CliError> {
    let io = |what: &'static str| move |e| CliError::Io(what.to_string(), e);
    let (ll, air, lib) = (
        dir.join("kernels.ll"),
        dir.join("kernels.air"),
        dir.join("kernels.metallib"),
    );
    std::fs::write(&ll, air_ir).map_err(io("writing AIR"))?;
    let mut metal = Command::new("xcrun");
    metal
        .args(["-sdk", "macosx", "metal", "-c", "-x", "ir"])
        .arg(&ll)
        .arg("-o")
        .arg(&air);
    run_tool(metal)?;
    let mut metallib = Command::new("xcrun");
    metallib
        .args(["-sdk", "macosx", "metallib"])
        .arg(&air)
        .arg("-o")
        .arg(&lib);
    run_tool(metallib)?;
    std::fs::read(&lib).map_err(io("reading metallib"))
}
