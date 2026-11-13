// OnceLock::get_or_init whose init panics sets nothing: get stays None,
// a by-ref env drops while unwinding, other threads retry, LazyLock poisons.
use std::sync::{Arc, LazyLock, OnceLock};

struct Loud(&'static str);
impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

static CELL: OnceLock<String> = OnceLock::new();
static LAZY: LazyLock<u32> = LazyLock::new(|| panic!("lazy init"));

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let guard = Loud("by-ref env");
    let r = std::panic::catch_unwind(|| {
        CELL.get_or_init(|| {
            let _inner = Loud("inner");
            if guard.0.len() > 1 {
                panic!("first");
            }
            String::new()
        })
        .len()
    });
    println!("{} {:?}", r.is_err(), CELL.get());
    let owned = Loud("moved env");
    let r = std::panic::catch_unwind(move || {
        CELL.get_or_init(move || {
            let _o = &owned;
            panic!("second")
        })
        .len()
    });
    println!("{} {:?}", r.is_err(), CELL.get());
    let shared = Arc::new(OnceLock::new());
    let hs: Vec<_> = (0..4u32)
        .map(|i| {
            let s = shared.clone();
            std::thread::spawn(move || {
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    *s.get_or_init(|| {
                        if i % 2 == 0 {
                            panic!("even")
                        }
                        7u32
                    })
                }));
                r.ok()
            })
        })
        .collect();
    let mut got: Vec<Option<u32>> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    got.sort();
    println!("{:?} {:?}", shared.get().is_some(), got.iter().filter(|g| **g == Some(7)).count() >= 1);
    for _ in 0..2 {
        let r = std::panic::catch_unwind(|| *LAZY);
        println!("{}", r.is_err());
    }
    println!("{}", CELL.get_or_init(|| "set".into()));
}
