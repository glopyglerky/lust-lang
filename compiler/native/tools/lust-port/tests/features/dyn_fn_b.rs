// feature: dyn.fn
// form: &mut dyn FnMut params forwarded between fns, as in ts-rust program.rs callbacks
fn collect(f: &mut dyn FnMut(i32) -> i32) -> i32 { f(1) + f(2) }
fn outer(f: &mut dyn FnMut(i32) -> i32) -> i32 { collect(f) }
fn main() {
    let mut add = |n: i32| n + 100;
    println!("{}", outer(&mut add));
}
