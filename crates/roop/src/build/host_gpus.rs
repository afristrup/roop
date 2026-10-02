use roop_syntax::Target;

/// GPUs this machine can run, in preference order.
pub fn host_gpus() -> Vec<Target> {
    if cfg!(target_os = "macos") {
        vec![Target::Metal]
    } else if crate::find_tool("nvidia-smi").is_some() {
        vec![Target::Nvptx]
    } else {
        Vec::new()
    }
}
