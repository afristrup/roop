use std::path::PathBuf;

/// The runtime static library: `$ROOP_RT_LIB`, else next to this binary.
pub fn runtime_lib() -> PathBuf {
    if let Ok(path) = std::env::var("ROOP_RT_LIB") {
        return PathBuf::from(path);
    }
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent()
        .map(|dir| dir.join("libroop_rt.a"))
        .unwrap_or_default()
}
