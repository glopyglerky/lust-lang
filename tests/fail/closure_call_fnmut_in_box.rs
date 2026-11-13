// expect: L0110
// rustc E0525: a call through a Box takes `Fn`; this closure is only FnMut.
fn main() {
    let mut n = 0u32;
    let mut m = Box::new(|| {
        n += 1;
        n
    });
    m();
    println!("{}", n);
}
