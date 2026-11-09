// expect: L0104
// rustc E0599: a closure that captures by `&mut` is not Clone.
fn main() {
    let mut n = 0;
    let mut inc = || n += 1;
    let mut inc2 = inc.clone();
    inc();
    inc2();
    println!("{n}");
}
