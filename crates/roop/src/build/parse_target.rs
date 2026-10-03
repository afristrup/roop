use roop_syntax::Target;

pub fn parse_target(name: &str) -> Option<Target> {
    match name {
        "cpu" => Some(Target::Cpu),
        "metal" => Some(Target::Metal),
        "cuda" => Some(Target::Cuda),
        _ => None,
    }
}
