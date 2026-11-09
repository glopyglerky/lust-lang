// expect: L0303
fn main() {
    let mut x = 1;
    let mut inc = || x += 1;
    println!("{}", x);
    inc();
}
