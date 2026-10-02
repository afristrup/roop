/// What `roop build` writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emit {
    /// Textual LLVM IR, plus the device modules when there are any.
    Ir,
    /// A relocatable object with the GPU code embedded.
    Object,
    /// A linked executable; needs `--link` for a `main`.
    Executable,
}
