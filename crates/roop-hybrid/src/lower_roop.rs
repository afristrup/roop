use crate::{Binding, HybridError, HybridProgram, Layout, Lowerer, roop_block, unsupported};
use roop_syntax::{Item, Program, Span};

/// A checked roop program as a hybrid program: `main`, with every call
/// inlined, `uncall` as the adjoint, and roop's loops and branches as the
/// host's. `main` takes nothing, or the exit status as an int.
pub fn lower_roop(program: &Program) -> Result<HybridProgram, HybridError> {
    let fns = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(f) => Some((f.name.as_str(), f)),
            _ => None,
        })
        .collect::<std::collections::HashMap<_, _>>();
    let main = *fns
        .get("main")
        .ok_or_else(|| unsupported("a program without `main`", Span::from(0..0)))?;
    let mut l = Lowerer {
        fns,
        irreversible: roop_check::irreversible_fns(program),
        layout: Layout::default(),
        frame: Vec::new(),
        depth: 0,
    };
    for p in &main.params {
        let reg = Lowerer::fresh(&mut l.layout.ints, &p.name);
        l.frame.push((p.name.clone(), Binding::Int(reg)));
    }
    let mut ops = Vec::new();
    roop_block(&mut l, &main.body, false, &mut ops)?;
    Ok(HybridProgram {
        layout: l.layout,
        ops,
    })
}
