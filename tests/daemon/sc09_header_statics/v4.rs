use std::sync::{LazyLock, OnceLock};

pub fn ident<T: Copy>(t: T) -> T {
    t
}

pub fn count(a: u32) -> usize where String: Copy {
    if cfg!(target_family = "wasm") {
        return 0;
    }
    static SET: OnceLock<Option<usize>> = OnceLock::new();
    let set = *SET.get_or_init(|| std::env::var("LUST_NOPE_X").ok().and_then(|v| v.parse::<usize>().ok()));
    static NAMES: LazyLock<Vec<&'static str>> = LazyLock::new(|| vec!["a", "b"]);
    set.unwrap_or(a as usize + NAMES.len())
}

pub fn other(b: u32) -> u64 {
    static K: u64 = 9;
    b as u64 + K
}

fn main() {
    println!("{} {} {}", count(3), other(4), ident(1u8));
}
