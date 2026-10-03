use roop_syntax::Behaviour;
use std::collections::HashSet;

/// Why a behaviour is not well formed, or `Ok`: choices need branches with
/// distinct labels, variables must be bound, and recursion must be guarded by
/// a choice (`rec x { x }` never does anything).
pub fn well_formed(behaviour: &Behaviour) -> Result<(), String> {
    check(behaviour, &mut Vec::new())
}

fn check<'a>(behaviour: &'a Behaviour, bound: &mut Vec<&'a str>) -> Result<(), String> {
    match behaviour {
        Behaviour::End => Ok(()),
        Behaviour::Var(name) if bound.contains(&name.as_str()) => Ok(()),
        Behaviour::Var(name) => Err(format!("`{name}` is not bound by an enclosing rec")),
        Behaviour::Rec(name, body) => {
            if !guarded(body) {
                return Err(format!("recursion on `{name}` is not guarded by a choice"));
            }
            bound.push(name);
            let result = check(body, bound);
            bound.pop();
            result
        }
        Behaviour::Choice { branches, .. } => {
            if branches.is_empty() {
                return Err("a choice needs at least one branch".into());
            }
            let mut seen = HashSet::new();
            if let Some((label, _)) = branches.iter().find(|(label, _)| !seen.insert(label)) {
                return Err(format!("the label `{label}` appears twice in one choice"));
            }
            branches.iter().try_for_each(|(_, next)| check(next, bound))
        }
    }
}

fn guarded(body: &Behaviour) -> bool {
    match body {
        Behaviour::Var(_) => false,
        Behaviour::Rec(_, inner) => guarded(inner),
        _ => true,
    }
}
