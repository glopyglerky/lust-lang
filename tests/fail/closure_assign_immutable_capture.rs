// expect: L0308
fn main() {
    let x = 0;
    let mut c = || x = 5;
    c();
    println!("{}", x);
}
