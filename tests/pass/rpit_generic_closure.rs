// A generic fn that returns a closure as `impl Fn`: each instance has
// its own closure, also when two instances capture the same types
// (hunt2 generic_rpit_closure_no_body). The hidden closures are not
// Copy (lust rejects a Copy hidden type, L0602).
use std::fmt::Display;
fn wrap<T: Display>(t: T) -> impl Fn() -> String { let pre = String::from("<"); move || format!("{}{}>", pre, t) }
fn plus_one<F: Fn(u32) -> u32>(f: F) -> impl Fn(u32) -> u32 { move |x| f(x) + 1 }
fn compose(f: impl Fn(u32) -> u32, g: impl Fn(u32) -> u32) -> impl Fn(u32) -> u32 { move |x| g(f(x)) }
fn show<T: Default + Display>(tag: String) -> impl Fn() -> String { move || format!("{}{}", tag, T::default()) }
fn evens<T: Copy + Into<u64>>(v: Vec<T>) -> impl Iterator<Item = u64> { v.into_iter().map(|x| x.into()).filter(|x| x % 2 == 0) }
fn nested<T: Display + Clone>(t: T) -> impl Fn() -> String { move || { let u = t.clone(); let g = move || format!("{}{}", u, u); g() } }
fn main() {
    let h = wrap(5u32);
    println!("{}", h());
    let s = wrap(String::from("x"));
    println!("{}", s());
    let k = String::from("abc");
    let p = plus_one(move |x| x * k.len() as u32);
    println!("{}", p(4));
    let (a, b) = (String::from("a"), String::from("bb"));
    let c = compose(move |x| x + a.len() as u32, move |x| x * 10 + b.len() as u32);
    println!("{}", c(2));
    println!("{}", show::<u32>(String::from("n"))());
    let mut n = 0;
    for x in evens(vec![1u8, 2, 3, 4]) {
        n += x;
    }
    println!("{}", n);
    println!("{}", nested(String::from("z"))());
    more();
}
fn call_twice<F: Fn() -> String>(f: F) -> String { format!("{}{}", f(), f()) }
fn more() {
    let a = wrap(1u8);
    let b = wrap(2u8);
    println!("{} {}", a(), b());
    println!("{}", call_twice(wrap('q')));
    let v = vec![wrap(3i64), wrap(4i64)];
    println!("{}", v[1]());
}
