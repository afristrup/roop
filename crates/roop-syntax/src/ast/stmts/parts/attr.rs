use crate::Target;

/// `target` is `None` until the optimizer picks one or the program forces it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attr {
    Parallel {
        target: Option<Target>,
    },
    /// On a block statement: adjacent `#[concurrent]` blocks run as tasks of
    /// one group, communicating only through channels.
    Concurrent,
    /// On an ancilla, written `auto ancilla`: the compiler adds the `keep`s
    /// that let go of it. With a region, `auto<'r> ancilla`, it is also let go
    /// of at the end of each run of the statement labeled `'r`.
    Auto {
        region: Option<String>,
    },
    /// `'name: stmt`, the name of a loop or block that an `auto` can refer to.
    Label(String),
}
