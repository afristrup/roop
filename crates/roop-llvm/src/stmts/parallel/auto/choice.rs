use roop_syntax::Target;

/// Where a bare `#[parallel]` loop runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Choice {
    Cpu,
    Gpu(Target),
    /// The trip count is only known at run time: use the GPU from
    /// `break_even` iterations up, CPU threads below.
    Runtime {
        gpu: Target,
        break_even: i64,
    },
}
