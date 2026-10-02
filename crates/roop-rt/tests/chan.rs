use roop_rt::*;
use std::ffi::c_void;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use std::time::Duration;

fn send(ch: *mut Channel, value: i64) {
    unsafe { roop_chan_send(ch, &value as *const i64 as *const u8) };
}

fn recv(ch: *mut Channel, abort: *const i32) -> Result<i64, ()> {
    let mut value = 0i64;
    match unsafe { roop_chan_recv(ch, &mut value as *mut i64 as *mut u8, abort) } {
        0 => Ok(value),
        _ => Err(()),
    }
}

#[derive(Clone, Copy)]
struct Ptr(*mut Channel);
unsafe impl Send for Ptr {}

#[test]
fn messages_arrive_in_order_across_threads() {
    let ch = Ptr(roop_chan_new(8));
    let producer = thread::spawn(move || {
        let ch = ch;
        (0..1000).for_each(|i| send(ch.0, i));
    });
    let got: Vec<i64> = (0..1000)
        .map(|_| recv(ch.0, std::ptr::null()).unwrap())
        .collect();
    producer.join().unwrap();
    assert_eq!(got, (0..1000).collect::<Vec<_>>());
    assert_eq!(unsafe { roop_chan_len(ch.0) }, 0);
    unsafe { roop_chan_free(ch.0) };
}

#[test]
fn a_raised_abort_flag_releases_a_blocked_receiver() {
    let ch = Ptr(roop_chan_new(8));
    let abort = Box::leak(Box::new(AtomicI32::new(0)));
    let flag = abort as *const AtomicI32 as usize;
    let waiter = thread::spawn(move || {
        let ch = ch;
        recv(ch.0, flag as *const i32)
    });
    thread::sleep(Duration::from_millis(20));
    abort.store(1, Ordering::Relaxed);
    assert_eq!(waiter.join().unwrap(), Err(()));
    unsafe { roop_chan_free(ch.0) };
}

#[test]
fn restoring_a_snapshot_brings_back_consumed_and_drops_new_messages() {
    let ch = roop_chan_new(8);
    send(ch, 1);
    send(ch, 2);
    let snapshot = unsafe { roop_chan_snapshot(ch) };
    assert_eq!(recv(ch, std::ptr::null()), Ok(1));
    send(ch, 99);
    unsafe { roop_chan_restore(ch, snapshot) };
    assert_eq!(unsafe { roop_chan_len(ch) }, 2);
    assert_eq!(recv(ch, std::ptr::null()), Ok(1));
    assert_eq!(recv(ch, std::ptr::null()), Ok(2));
    unsafe {
        roop_chan_snapshot_free(snapshot);
        roop_chan_free(ch);
    }
}

extern "C" fn bump(env: *mut c_void, _: i64) {
    unsafe { &*(env as *const std::sync::atomic::AtomicI64) }.fetch_add(1, Ordering::Relaxed);
}

#[test]
fn concurrent_runs_every_task_and_waits_for_all() {
    let counter = std::sync::atomic::AtomicI64::new(0);
    let tasks = [bump as extern "C" fn(*mut c_void, i64); 5];
    let envs = [&counter as *const _ as *mut c_void; 5];
    unsafe { roop_concurrent(5, tasks.as_ptr(), envs.as_ptr()) };
    assert_eq!(counter.load(Ordering::Relaxed), 5);
}
