use roop_syntax::Type;

#[derive(Clone, Debug)]
pub struct Value {
    pub reg: String,
    pub ty: Type,
}
