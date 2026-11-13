// K1 compatibility fixture: a recovered panic (catch_unwind) with drops.
// panic-mode: unwind
//
// Real shape: crates/ts_goport/src/program.rs job runners wrap work
// in std::panic::catch_unwind(AssertUnwindSafe(f)) and send the
// Result on; the bind loop also catches per file. GoPanic payloads
// in ts-rust are a struct; here the payloads are &'static str and
// String, the two std forms. Requires panic=unwind: under abort the
// process dies at the first panic and this fixture cannot run.
//
// The panic hook is silenced so stderr does not depend on panic
// formatting (ts-rust installs its own Go panic hook). Stdout is
// the record. Drop order during unwind is the point: guards print
// through a thread-local log so order is observable after catch.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

std::thread_local! {
    static LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn note(text: &str) {
    LOG.with(|log| log.borrow_mut().push(text.to_string()));
}

/// A scope guard, like a checker trace or a file pin release.
struct Guard(&'static str);

impl Guard {
    fn new(name: &'static str) -> Guard {
        note(&format!("open {name}"));
        Guard(name)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        note(&format!("close {}", self.0));
    }
}

fn payload_text(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        format!("str:{s}")
    } else if let Some(s) = payload.downcast_ref::<String>() {
        format!("string:{s}")
    } else {
        "other".to_string()
    }
}

/// A job like the emit pool runs: catch, report, keep going.
fn run_job(name: &'static str, f: impl FnOnce() -> i32) {
    let result = catch_unwind(AssertUnwindSafe(f));
    match result {
        Ok(value) => println!("job {name}: ok {value}"),
        Err(payload) => println!("job {name}: panicked {}", payload_text(&payload)),
    }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));

    run_job("plain", || {
        let _g = Guard::new("outer");
        40 + 2
    });

    run_job("str-panic", || {
        let _g1 = Guard::new("first");
        let _g2 = Guard::new("second");
        let v = vec![1, 2, 3];
        note(&format!("vec ready {}", v.len()));
        panic!("bind failed")
    });

    run_job("string-panic", || {
        let _g = Guard::new("only");
        // panic_any with a String: the payload downcast takes the
        // String arm (a formatted panic! may stay a &'static str).
        std::panic::panic_any(format!("file {} broke", 7))
    });

    // Nested catch: the inner panic is recovered, the outer job
    // still completes (ts-rust binds the next file either way).
    run_job("nested", || {
        let _g = Guard::new("outer");
        let inner = catch_unwind(AssertUnwindSafe(|| {
            let _g = Guard::new("inner");
            panic!("inner bind failed")
        }));
        note(&format!("inner recovered: {}", inner.is_err()));
        5
    });

    // A poisoned lock after a panic while holding it: the standard
    // recovery reads the value back out of the poison error.
    let lock = Mutex::new(vec![1, 2, 3]);
    let poisoned = catch_unwind(AssertUnwindSafe(|| {
        let mut data = lock.lock().unwrap();
        data.push(4);
        panic!("poison now")
    }));
    println!("poison job panicked: {}", poisoned.is_err());
    match lock.lock() {
        Ok(_) => println!("lock clean (unexpected)"),
        Err(poison) => {
            let data = poison.into_inner();
            println!("lock poisoned, data kept: {:?}", *data);
        }
    }

    LOG.with(|log| {
        for line in log.borrow().iter() {
            println!("log: {line}");
        }
    });
}
