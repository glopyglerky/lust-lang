// expect: L0308
fn main() {
    let mut n = 0;
    let bump = || n += 1;
    bump();
    println!("{}", n);
}
