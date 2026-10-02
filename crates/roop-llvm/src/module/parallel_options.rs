use crate::CostModel;
use roop_syntax::Target;

/// How `#[parallel]` loops are placed.
#[derive(Clone, Debug)]
pub struct ParallelOptions {
    /// Targets a loop may name explicitly. Naming another is an error.
    pub allowed: Vec<Target>,
    /// GPUs the compiler may pick for a bare `#[parallel]`, in preference
    /// order. Empty means bare loops always use CPU threads.
    pub auto_gpus: Vec<Target>,
    pub cost: CostModel,
}

impl Default for ParallelOptions {
    fn default() -> Self {
        ParallelOptions {
            allowed: vec![Target::Cpu, Target::Metal, Target::Nvptx],
            auto_gpus: Vec::new(),
            cost: CostModel::default(),
        }
    }
}
