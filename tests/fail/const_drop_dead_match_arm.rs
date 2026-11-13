// expect: L0631
// rustc E0493: the drop check is static; a dead match arm still counts.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
const X: u32 = match Some(3u32) {
    Some(n) => n,
    None => {
        let _d = D(1);
        0
    }
};
fn main() { println!("{}", X); }
