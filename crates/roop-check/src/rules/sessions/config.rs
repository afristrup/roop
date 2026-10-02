use roop_syntax::Behaviour;

/// A configuration of a client and a server: each with the behaviour it would
/// return to on a rollback (its last checkpoint, if it passed one) and the
/// behaviour it is running.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    pub client_past: Option<Behaviour>,
    pub client: Behaviour,
    pub server_past: Option<Behaviour>,
    pub server: Behaviour,
}
