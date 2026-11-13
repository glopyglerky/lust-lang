// `sum`/`product` whose result type only a later use fixes: rustc keeps
// `S: Sum<Item>` pending (ts-rust: `ARENA.with(|a| .. .map(Vec::len).sum())`).
use std::cell::RefCell;

struct Arena {
    slots: Vec<Option<Vec<u8>>>,
}

thread_local! {
    static ARENA: RefCell<Arena> = RefCell::new(Arena { slots: vec![Some(vec![1, 2]), None, Some(vec![3])] });
}

fn live() -> usize {
    ARENA.with(|a| a.borrow().slots.iter().flatten().map(Vec::len).sum())
}

fn total(v: &Vec<u64>) -> u64 {
    Some(v).map(|v| v.iter().sum()).unwrap_or(0)
}

fn main() {
    let v = vec![1i32, 2, 3];
    // Fixed only as a literal: i32 after fallback, and `i32: Sum<&i32>` holds.
    let r = Some(&v).map(|v| v.iter().sum()).unwrap_or(0);
    let p = v.iter().product();
    let q: i32 = p;
    let w = vec![1.5f64, 2.0];
    let f = Some(&w).map(|w| w.iter().copied().product()).unwrap_or(1.0);
    println!("{} {} {} {} {}", live(), total(&vec![4, 5]), r, q, f);
}
