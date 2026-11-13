// expect: L0631
// rustc E0493: an `if let` body that never runs drops its local.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
const N: Option<u32> = None;
const X: u32 = if let Some(n) = N {
    let _d = D(n);
    1
} else {
    0
};
fn main() { println!("{}", X); }
