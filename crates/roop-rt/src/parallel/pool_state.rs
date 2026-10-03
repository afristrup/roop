use crate::parallel::Job;

pub struct PoolState {
    pub generation: u64,
    pub job: Option<Job>,
    pub running: usize,
}
