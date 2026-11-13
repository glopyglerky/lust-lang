// expect: L0303
// rustc E0499: the spare view borrows the Vec mutably while push runs.
fn main() {
    let mut v: Vec<u8> = Vec::with_capacity(4);
    let s = v.spare_capacity_mut();
    v.push(1);
    s[0].write(2);
}
