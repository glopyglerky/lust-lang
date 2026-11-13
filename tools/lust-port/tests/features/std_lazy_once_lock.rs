// feature: std.lazy_once_lock
// form: static LazyLock and OnceLock with get_or_init, as in ts-rust encoder tables
use std::sync::{LazyLock, OnceLock};
static TABLE: LazyLock<Vec<i32>> = LazyLock::new(|| vec![1, 2, 3]);
static ONE: OnceLock<i32> = OnceLock::new();
fn main() { println!("{} {}", TABLE.len(), ONE.get_or_init(|| 9)); }
