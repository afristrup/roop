use roop_syntax::Place;

pub enum Binding {
    Local(String),
    Alias(String, Place),
}
