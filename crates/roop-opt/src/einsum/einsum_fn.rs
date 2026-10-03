use crate::{Spec, length_name, number_spans};
use roop_syntax::{Block, FnDef, Param, Span, Stmt};

/// A reversible function with a length for each label of the spec.
pub fn einsum_fn(
    name: &str,
    public: bool,
    spec: &Spec,
    params: Vec<Param>,
    mut stmts: Vec<Stmt>,
) -> FnDef {
    number_spans(&mut stmts, &mut 1);
    FnDef {
        name: name.into(),
        generics: spec.labels().into_iter().map(length_name).collect(),
        params,
        body: Block {
            stmts,
            span: Span::from(0..0),
        },
        irreversible: false,
        public,
        test: false,
        bennett: None,
        external: false,
        world: false,
        einsum: None,
    }
}
