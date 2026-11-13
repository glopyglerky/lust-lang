// Atomic*::fetch_update (std's load + compare-exchange loop; Err(prev) when the
// closure gives None) and Condvar::wait_timeout_while (std's checked_sub loop).
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
fn atomics() {
    let a = AtomicU64::new(7);
    println!("{:?}", a.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |c| c.checked_add(1)));
    println!("{:?} {}", a.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| None), a.load(Ordering::Relaxed));
    let m = AtomicU64::new(u64::MAX);
    println!("{:?}", m.fetch_update(Ordering::AcqRel, Ordering::Acquire, |c| c.checked_add(1)));
    let b = AtomicBool::new(false);
    println!("{:?} {:?}", b.fetch_update(Ordering::Release, Ordering::Relaxed, |x| Some(!x)), b.load(Ordering::Relaxed));
    let i = AtomicI8::new(-3);
    let mut calls = 0;
    let r = i.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| { calls += 1; if x < 0 { Some(x * 2) } else { None } });
    println!("{:?} {} {}", r, i.load(Ordering::Relaxed), calls);
    let u = AtomicUsize::new(1);
    let s = String::from("cap");
    let r = u.fetch_update(Ordering::Relaxed, Ordering::Relaxed, move |x| Some(x + s.len()));
    println!("{:?} {}", r, u.into_inner());
}

fn condvar() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let p2 = Arc::clone(&pair);
    let h = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        let (m, c) = &*p2;
        *m.lock().unwrap() = true;
        c.notify_all();
    });
    let (m, c) = &*pair;
    let g = m.lock().unwrap();
    let (g, r) = c.wait_timeout_while(g, Duration::from_secs(5), |done| !*done).unwrap();
    println!("{} {}", *g, r.timed_out());
    drop(g);
    h.join().unwrap();
    let mut n = 0;
    let g = m.lock().unwrap();
    let (g, r) = c.wait_timeout_while(g, Duration::from_millis(20), |_| { n += 1; true }).unwrap();
    println!("{} {} {}", *g, r.timed_out(), n >= 1);
    let (g, r) = c.wait_timeout_while(g, Duration::ZERO, |_| false).unwrap();
    println!("{} {}", *g, r.timed_out());
    let (g, r) = c.wait_timeout_while(g, Duration::ZERO, |_| true).unwrap();
    println!("{} {}", *g, r.timed_out());
    let res = c.wait_timeout_while(g, Duration::from_millis(1), |v| { *v = false; true });
    let (g, r) = res.unwrap();
    println!("{} {}", *g, r.timed_out());
}

fn main() {
    atomics();
    condvar();
}
