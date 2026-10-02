use roop_syntax::Target;

pub fn parse_target(name: &str) -> Option<Target> {
    match name {
        "cpu" => Some(Target::Cpu),
        "metal" => Some(Target::Metal),
        "nvptx" => Some(Target::Nvptx),
        _ => None,
    }
}
