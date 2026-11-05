use crate::CliError;
use roop_syntax::{Expr, Item, Place, StmtKind, parse};
use roop_weave::Model;

pub fn validate_weave_certificate(source: &str, model: &Model) -> Result<(), CliError> {
    let program = parse(source)
        .map_err(|error| CliError::Tool(format!("cannot parse generated weave source: {error}")))?;
    let name = format!("{}_forward", model.name);
    let forward = program
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(function) if function.name == name => Some(function),
            _ => None,
        })
        .ok_or_else(|| {
            CliError::Tool(format!("generated weave source has no `{name}` function"))
        })?;
    let direction = |expr: &Expr| match expr {
        Expr::Place(Place::Var(name)) if name == "q" => Ok(true),
        Expr::Place(Place::Var(name)) if name == "p" => Ok(false),
        _ => Err(CliError::Tool(
            "generated weave coupling calls must target `q` or `p`".into(),
        )),
    };
    let mut actual = Vec::new();
    for statement in &forward.body.stmts {
        let StmtKind::Call { callee, args, .. } = &statement.kind else {
            return Err(CliError::Tool(
                "generated weave forward function contains a non-coupling statement".into(),
            ));
        };
        match callee.as_str() {
            "layer" => {
                let first = args.first().ok_or_else(|| {
                    CliError::Tool("generated `layer` call has no target argument".into())
                })?;
                let second = args.get(1).ok_or_else(|| {
                    CliError::Tool("generated `layer` call has no source argument".into())
                })?;
                if !direction(first)? || direction(second)? {
                    return Err(CliError::Tool(
                        "generated `layer` call does not preserve `p` while updating `q`".into(),
                    ));
                }
                actual.extend([false, true, false]);
            }
            "mlp" | "mlp_norm" | "attn" | "conv" => {
                let source_index = match callee.as_str() {
                    "mlp" => 5,
                    "mlp_norm" => 6,
                    "attn" => 4,
                    "conv" => 3,
                    _ => unreachable!(),
                };
                let target = args.first().ok_or_else(|| {
                    CliError::Tool(format!("generated `{callee}` call has no target argument"))
                })?;
                let source = args.get(source_index).ok_or_else(|| {
                    CliError::Tool(format!("generated `{callee}` call has no source argument"))
                })?;
                let target = direction(target)?;
                if target == direction(source)? {
                    return Err(CliError::Tool(format!(
                        "generated `{callee}` call does not preserve the source tensor"
                    )));
                }
                actual.push(target);
            }
            _ => {
                return Err(CliError::Tool(format!(
                    "generated weave forward function has unexpected call `{callee}`"
                )));
            }
        }
    }
    let expected = model.certificate_directions();
    if actual == expected {
        Ok(())
    } else {
        Err(CliError::Tool(format!(
            "generated weave directions {actual:?} differ from certificate directions {expected:?}"
        )))
    }
}
