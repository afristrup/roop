use roop_syntax::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Construct {
    Loop,
    Try,
}

#[derive(Clone, Debug)]
pub struct AdditiveUpdate {
    pub delta: String,
    pub index: String,
    pub index_is_state: bool,
}

#[derive(Clone, Debug)]
pub struct ScalarAdditiveUpdate;

/// A loop or `try` whose parts were lifted into top-level definitions: the
/// variables they read (`captures`) and the tuple of variables they write
/// (`state`).
#[derive(Clone, Debug)]
pub struct Lifted {
    pub id: String,
    pub construct: Construct,
    pub captures: Vec<(String, Type)>,
    pub state: Vec<(String, Type)>,
    /// The loop variable of a `#[parallel]` loop.
    pub parallel: Option<String>,
    /// Holds a `try` somewhere inside. A `try` is undone only on states it
    /// produced, so the pieces are inverses in one direction only.
    pub one_way: bool,
    pub additive_update: Option<AdditiveUpdate>,
    pub scalar_additive_update: Option<ScalarAdditiveUpdate>,
}
