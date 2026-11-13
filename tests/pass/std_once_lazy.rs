// `std::cell::{OnceCell, LazyCell}` and `std::sync::{OnceLock, LazyLock}`
// are lust std source (library/core/src/cell.rs, library/std/src/sync.rs,
// W-STD-C) over the `once_init` / `once_take` builtins: statics of
// `OnceLock`, get_or_init, set, take, clone, eq, `LazyLock::force` (init
// runs once), and the exact Debug text, plain and pretty.
use std::cell::OnceCell;
use std::sync::{LazyLock, OnceLock};
static ONE: OnceLock<i32> = OnceLock::new();
static NAMES: OnceLock<Vec<String>> = OnceLock::new();
fn names() -> &'static Vec<String> {
    NAMES.get_or_init(|| vec!["a".to_string(), "b".to_string()])
}
fn main() {
    println!("{:?}", ONE.get());
    println!("{}", ONE.get_or_init(|| 9));
    println!("{}", ONE.get_or_init(|| 10));
    println!("{:?} {:?}", ONE.set(5), ONE);
    println!("{:?} {}", names(), names().len());
    let c: OnceCell<String> = OnceCell::new();
    println!("{:?}", c);
    println!("{}", c.get_or_init(|| "x".to_string()));
    println!("{:?} {:?}", c, c.set("y".to_string()));
    let mut c2 = c.clone();
    println!("{} {:?} {:?}", c == c2, c2.take(), c2);
    let l = LazyLock::new(|| { println!("init"); 42 });
    println!("before {:?}", l);
    println!("{}", LazyLock::force(&l));
    println!("{} {:?}", LazyLock::force(&l), l);
    println!("{:#?}", c);
    let lc = std::cell::LazyCell::new(|| 5);
    println!("{:?} {} {:?}", std::cell::LazyCell::get(&lc), *std::cell::LazyCell::force(&lc), lc);
    let mut lm = LazyLock::new(|| vec![1]);
    LazyLock::force_mut(&mut lm).push(2);
    println!("{:?} {:?}", LazyLock::get(&lm), lm);
    let from: OnceLock<u8> = OnceLock::from(3);
    let mut empty: OnceLock<u8> = OnceLock::default();
    println!("{:?} {:?}", from.get(), empty.get_mut());
    println!("{:?}", from.into_inner());
    println!("{:#?}", OnceCell::<u8>::new());
}
