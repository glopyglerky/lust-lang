// expect: L0306
// rustc E0597: the captured `Option<&u32>` borrows a local, so the closure is not `'static`.
fn keep(f: Box<dyn Fn() -> u32>) -> u32 { f() }
fn main() {
    let n = 5u32;
    let x: Option<&u32> = Some(&n);
    println!("{}", keep(Box::new(move || x.map_or(0, |v| *v))));
}
