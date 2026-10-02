/// Why a pair of behaviours is not compliant, with the actions taken to get
/// to the configuration where it goes wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub path: Vec<String>,
    pub reason: String,
}
