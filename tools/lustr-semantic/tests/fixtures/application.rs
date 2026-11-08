use std::num::NonZeroU64;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};

static DROPS: AtomicUsize = AtomicUsize::new(0);

struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[inline(never)]
fn generic<T>(value: T) -> T {
    bridge::identity(value)
}

#[inline(never)]
fn guarded(should_panic: bool) -> u64 {
    let _guard = Guard;
    if should_panic {
        panic!("expected panic");
    }
    7
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let big = bridge::large(bridge::Large([2; 6]));
    assert_eq!(big, bridge::Large([2, 2, 2, 2, 2, 3]));
    assert_eq!(bridge::slice(&big.0), &[2, 2, 2, 2, 3]);
    assert_eq!(bridge::niche(NonZeroU64::new(4)), 4);
    assert_eq!(bridge::niche(None), 0);
    assert_eq!(generic(10_u32), 10);
    assert_eq!(generic(20_u64), 20);
    assert_eq!(bridge::invoke(&mut || guarded(false)), 7);
    assert!(catch_unwind(AssertUnwindSafe(|| guarded(true))).is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 2);
    println!("aggregate=3 drops=2 generics=30");
}
