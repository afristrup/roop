/// Everything a program compiles to. `air` and `ptx` exist only when some
/// `#[parallel]` loop was sent to that GPU.
pub struct Compiled {
    pub host: String,
    pub air: Option<String>,
    pub ptx: Option<String>,
}
