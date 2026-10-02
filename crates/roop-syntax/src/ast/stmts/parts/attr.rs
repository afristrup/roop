use crate::Target;

/// `target` is `None` until the optimizer picks one or the program forces it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attr {
    Parallel { target: Option<Target> },
}
