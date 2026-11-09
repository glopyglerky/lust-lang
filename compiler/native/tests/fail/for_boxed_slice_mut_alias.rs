// expect: L0303
// rustc E0502: the loop holds `&mut b`, so `b.len()` inside it is a second borrow.
fn main() {
    let mut b: Box<[u8]> = vec![1, 2].into_boxed_slice();
    for x in &mut b {
        let n = b.len();
        *x += n as u8;
    }
}
