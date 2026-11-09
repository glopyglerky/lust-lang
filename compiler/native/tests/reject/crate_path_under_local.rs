// expect: L0505
// `crate::f` under a local `f`: lust rejects it for now.
fn f() -> i32 { 1 }
fn main() {
    let f = || 5;
    println!("{} {}", f(), crate::f());
}
