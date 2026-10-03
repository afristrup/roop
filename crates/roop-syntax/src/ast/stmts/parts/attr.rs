use crate::Target;

/// `target` is `None` until the optimizer picks one or the program forces it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attr {
    Parallel {
        target: Option<Target>,
    },
    /// On a block statement: adjacent `#[concurrent]` blocks run as tasks of
    /// one group, communicating only through channels.
    Concurrent,
    /// On an ancilla, written `auto ancilla`: the compiler adds the `keep`s
    /// that let go of it, where it is no longer used.
    Auto,
}
