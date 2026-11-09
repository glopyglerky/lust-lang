// expect: L0304
fn main() {
    let mut n = 0;
    let mut a = || n += 1;
    let mut b = || n += 2;
    a();
    b();
}
