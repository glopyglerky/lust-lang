// A reference to a closure or fn item as the function of `catch_unwind`,
// `get_or_init` and `Arc::new_cyclic`: the closure's body takes the
// reference as its env. lust passed the address of the reference
// instead and read garbage captures.
use std::panic::catch_unwind;
use std::sync::OnceLock;

fn seven() -> i32 {
    7
}

fn main() {
    let k = 40;
    let f = move || k + 2;
    println!("{:?}", catch_unwind(&f).ok());
    println!("{:?}", catch_unwind(&seven).ok());
    let s = String::from("abc");
    let g = || s.len();
    let c: OnceLock<usize> = OnceLock::new();
    println!("{}", c.get_or_init(&g));
    let r = std::sync::Arc::new_cyclic(&|_w: &std::sync::Weak<i32>| 5 + k);
    println!("{}", r);
    let mut n = 1;
    let mut bump = || {
        n += 1;
        n
    };
    let once = OnceLock::new();
    println!("{}", once.get_or_init(&mut bump));
    println!("{}", once.get_or_init(&mut bump));
    println!("{n}");
}
