use crate::{Translation, esc, esc_fn, lean_type, lean_zero, tuple_expr, tuple_proj};
use roop_syntax::{Item, Program, Type};

/// Zero for a fixture: a number, a bool, an array of them, or an empty stack.
fn fixture_zero(ty: &Type) -> Option<String> {
    match ty {
        Type::Array(elem, len) => Some(format!("(Vector.replicate {len} {})", fixture_zero(elem)?)),
        Type::Stack(..) => Some(format!("(Roop.Stack.empty : {})", lean_type(ty))),
        other => lean_zero(other).ok(),
    }
}

/// For each test the model could translate, a Lean `#eval` that runs it on
/// zero fixtures, then runs the inverse on the result, and prints
/// `@lean <name> true` when the first passed every expectation and the second
/// gave the zero fixtures back. `roop test --lean` compares these lines with
/// what the compiled tests did.
pub fn test_checks(program: &Program, translation: &Translation) -> String {
    let mut out = String::new();
    for item in &program.items {
        let Item::Fn(def) = item else { continue };
        if !def.test || translation.skipped.iter().any(|(n, _)| *n == def.name) {
            continue;
        }
        let types: Option<Vec<Type>> = def
            .params
            .iter()
            .map(|p| match &p.ty {
                Type::Ref { inner, .. } => Some((**inner).clone()),
                _ => None,
            })
            .collect();
        let zeros: Option<Vec<String>> = types
            .as_ref()
            .and_then(|types| types.iter().map(fixture_zero).collect());
        let (Some(types), Some(zeros)) = (types, zeros) else {
            continue;
        };
        let n = types.len();
        let args = zeros.join(" ");
        let back_args: Vec<String> = (0..n).map(|k| tuple_proj("__out", k, n)).collect();
        let name = esc(&def.name);
        out.push_str(&format!(
            "def Check.{name} : Bool :=\n  match {} {args} with\n  | Except.ok __out =>\n    match {} {} with\n    | Except.ok __back => __back == {}\n    | Except.error _ => false\n  | Except.error _ => false\n#eval IO.println s!\"@lean {} {{Check.{name}}}\"\n",
            esc_fn(&def.name),
            esc_fn(&format!("{}_inv", def.name)),
            back_args.join(" "),
            tuple_expr(&zeros),
            def.name
        ));
    }
    out
}
