use roop_syntax::Type;

/// A variable's address and the type stored there.
#[derive(Clone, Debug)]
pub struct Slot {
    pub addr: String,
    pub ty: Type,
}
