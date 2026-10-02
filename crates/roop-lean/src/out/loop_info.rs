use roop_syntax::Type;

/// A loop whose pieces were lifted into top-level definitions: the variables
/// they read (`captures`) and the tuple of variables they write (`state`).
#[derive(Clone, Debug)]
pub struct LoopInfo {
    pub id: String,
    pub captures: Vec<(String, Type)>,
    pub state: Vec<(String, Type)>,
}
