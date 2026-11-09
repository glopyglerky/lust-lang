// capability: types: std::cell::{OnceCell, LazyCell} and std::sync::{OnceLock, LazyLock, Once} (std source; single-threaded; no Once::call_once_force)
// expect: ok

use std::cell::{LazyCell, OnceCell};
use std::sync::{LazyLock, Once, OnceLock};

static TABLE: LazyLock<Vec<u32>> = LazyLock::new(|| vec![1, 2, 3]);
static SET: OnceLock<u32> = OnceLock::new();

fn main() {
    let c = OnceCell::new();
    let lc = LazyCell::new(|| 40 + 2);
    let once = Once::new();
    let mut ran = 0;
    once.call_once(|| ran += 1);
    once.call_once(|| ran += 1);
    SET.set(5).unwrap();
    println!("{} {} {} {} {}", c.get_or_init(|| 1), *lc, TABLE[2], SET.get().unwrap(), ran);
}
