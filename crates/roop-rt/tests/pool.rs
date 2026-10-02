use roop_rt::roop_parallel_for;
use std::ffi::c_void;
use std::sync::atomic::{AtomicI64, Ordering};
use std::thread;

extern "C" fn add_value(env: *mut c_void, value: i64) {
    let sum = unsafe { &*(env as *const AtomicI64) };
    sum.fetch_add(value, Ordering::Relaxed);
}

fn run(lo: i64, count: i64, step: i64) -> i64 {
    let sum = AtomicI64::new(0);
    unsafe { roop_parallel_for(lo, count, step, add_value, &sum as *const _ as *mut c_void) };
    sum.load(Ordering::Relaxed)
}

fn expected(lo: i64, count: i64, step: i64) -> i64 {
    (0..count).map(|k| lo + k * step).sum()
}

#[test]
fn visits_every_iteration_exactly_once() {
    for (lo, count, step) in [
        (0, 1, 1),
        (0, 2, 1),
        (5, 1000, 3),
        (-7, 100_000, 2),
        (0, 0, 1),
    ] {
        assert_eq!(run(lo, count, step), expected(lo, count, step));
    }
}

#[test]
fn survives_thousands_of_back_to_back_loops() {
    for round in 0..5000 {
        assert_eq!(run(round, 64, 1), expected(round, 64, 1));
    }
}

extern "C" fn nested(env: *mut c_void, _value: i64) {
    let sum = unsafe { &*(env as *const AtomicI64) };
    let inner = AtomicI64::new(0);
    unsafe { roop_parallel_for(0, 10, 1, add_value, &inner as *const _ as *mut c_void) };
    sum.fetch_add(inner.load(Ordering::Relaxed), Ordering::Relaxed);
}

#[test]
fn nested_loops_run_sequentially_instead_of_deadlocking() {
    let sum = AtomicI64::new(0);
    unsafe { roop_parallel_for(0, 200, 1, nested, &sum as *const _ as *mut c_void) };
    assert_eq!(sum.load(Ordering::Relaxed), 200 * 45);
}

#[test]
fn concurrent_callers_all_get_correct_results() {
    let handles: Vec<_> = (0..8)
        .map(|t| {
            thread::spawn(move || {
                for round in 0..500 {
                    let (lo, count) = (t * 1000 + round, 500 + round);
                    assert_eq!(run(lo, count, 1), expected(lo, count, 1));
                }
            })
        })
        .collect();
    handles.into_iter().for_each(|h| h.join().unwrap());
}
