/// Which LLVM flavour is being written: the host, NVIDIA PTX, or Apple AIR.
/// AIR predates opaque pointers, so it needs typed pointer syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    Host,
    Nvptx,
    Air,
}
