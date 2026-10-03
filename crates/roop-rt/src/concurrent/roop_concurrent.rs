use crate::parallel::{Body, SendPtr};
use std::ffi::c_void;
use std::thread;

/// Runs `n` tasks at once and returns when all have finished. The calling
/// thread runs the first. Tasks may block on channels, so each gets its own
/// thread rather than a slot in the loop pool.
///
/// # Safety
/// `tasks` and `envs` hold `n` valid entries, and the tasks only share data
/// the compiler proved disjoint or reached through channels.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn roop_concurrent(n: i64, tasks: *const Body, envs: *const *mut c_void) {
    let (tasks, envs) = unsafe {
        (
            std::slice::from_raw_parts(tasks, n as usize),
            std::slice::from_raw_parts(envs, n as usize),
        )
    };
    thread::scope(|scope| {
        for (task, env) in tasks.iter().zip(envs).skip(1) {
            let (task, env) = (*task, SendPtr(*env));
            scope.spawn(move || {
                let env = env;
                task(env.0, 0);
            });
        }
        tasks[0](envs[0], 0);
    });
}
