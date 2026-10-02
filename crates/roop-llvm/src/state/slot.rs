use roop_syntax::Type;

/// A variable's address, the type stored there, and its address space
/// (0 is private or host memory, 1 is device buffer memory).
#[derive(Clone, Debug)]
pub struct Slot {
    pub addr: String,
    pub ty: Type,
    pub space: u32,
}
