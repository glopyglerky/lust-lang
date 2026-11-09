// expect: L0308
fn main() {
    let mut n = 0;
    let mut bump = || n += 1;
    let r = &bump;
    r();
    println!("{}", n);
}
