// expect: L0303
fn keep<F: Fn() -> usize>(f: F) -> F {
    f
}
fn main() {
    let mut s = String::from("a");
    let c = keep(|| s.len());
    s.push('b');
    println!("{}", c());
}
