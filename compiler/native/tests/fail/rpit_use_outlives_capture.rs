// expect: L0303
// rustc E0506: `'b: 'a` lets the hidden type hold the `&'b [u8]` borrow
// shortened to `'a`, so the value captures `b`'s borrow through `'a`.
fn pick<'a, 'b: 'a>(_x: &'a u8, y: &'b [u8]) -> impl Iterator<Item = &'a u8> + use<'a> {
    y.iter()
}
fn main() {
    let a = 1;
    let mut b = [2u8];
    let v = pick(&a, &b);
    b[0] = 3;
    let _ = &v;
    println!("{}", b[0]);
}
