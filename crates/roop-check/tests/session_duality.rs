use roop_check::{compliant, dual, well_formed};
use roop_syntax::{Behaviour, ChoiceKind};

struct Lcg(u64);

impl Lcg {
    fn next(&mut self, below: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % below
    }
}

/// A random well-formed behaviour: choices over a few labels, checkpoints here
/// and there, and recursion that always passes through a choice.
fn random(rng: &mut Lcg, depth: u32, vars: &[String]) -> Behaviour {
    if depth == 0 {
        return match vars.last() {
            Some(var) if rng.next(2) == 0 => Behaviour::Var(var.clone()),
            _ => Behaviour::End,
        };
    }
    match rng.next(6) {
        0 => Behaviour::End,
        1 => {
            let name = format!("x{}", vars.len());
            let mut bound = vars.to_vec();
            bound.push(name.clone());
            let body = random_choice(rng, depth, &bound);
            Behaviour::Rec(name, Box::new(body))
        }
        _ => random_choice(rng, depth, vars),
    }
}

fn random_choice(rng: &mut Lcg, depth: u32, vars: &[String]) -> Behaviour {
    let labels = ["a", "b", "c"];
    let count = 1 + rng.next(3) as usize;
    let branches = labels[..count]
        .iter()
        .map(|l| (l.to_string(), random(rng, depth - 1, vars)))
        .collect();
    Behaviour::Choice {
        kind: if rng.next(2) == 0 {
            ChoiceKind::Offer
        } else {
            ChoiceKind::Select
        },
        checkpoint: rng.next(3) == 0,
        branches,
    }
}

#[test]
fn every_behaviour_is_checkpoint_compliant_with_its_dual() {
    let mut rng = Lcg(7);
    let mut checked = 0;
    for _ in 0..4000 {
        let behaviour = random(&mut rng, 4, &[]);
        if well_formed(&behaviour).is_err() {
            continue;
        }
        compliant(&behaviour, &dual(&behaviour))
            .unwrap_or_else(|v| panic!("{behaviour:?} against its dual: {v:?}"));
        checked += 1;
    }
    assert!(checked > 3000, "only {checked} samples were well formed");
}

#[test]
fn the_dual_is_an_involution() {
    let mut rng = Lcg(11);
    for _ in 0..500 {
        let behaviour = random(&mut rng, 3, &[]);
        assert_eq!(dual(&dual(&behaviour)), behaviour);
    }
}
