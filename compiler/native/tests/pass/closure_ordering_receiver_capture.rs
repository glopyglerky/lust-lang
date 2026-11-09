// Ordering's then_with, then, reverse, and is_* take self by value: a
// receiver that reads a captured variable is a by-value use of a Copy value.
use std::cmp::Ordering;
fn want_fn<F: Fn(u32, u32) -> i32>(f: F) -> i32 { f(1, 2) + f(2, 1) }
fn want_fnmut<F: FnMut(u32, u32) -> i32>(mut f: F) -> i32 { f(1, 2) + f(2, 1) }
fn main() {
    let s = vec![5u32, 1, 2];
    let c = |left: u32, right: u32| -> i32 {
        Ordering::Equal.then_with(|| s[left as usize].cmp(&s[right as usize])).then_with(|| left.cmp(&right)) as i32
    };
    println!("{} {}", c(1, 2), want_fn(&c));
    let mut n = 0u32;
    let d = |a: u32, b: u32| -> i32 {
        (a.cmp(&b).then_with(|| { n += 1; n.cmp(&b) }).reverse()).is_lt() as i32
    };
    println!("{}", want_fnmut(d));
    println!("{}", n);
    let t = String::from("abc");
    let e = move |a: u32, _b: u32| -> i32 { (t.len().cmp(&(a as usize)).then(Ordering::Less)).is_ge() as i32 };
    println!("{}", want_fn(e));
}
