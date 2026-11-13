// expect: L0104
// rustc E0277: on an unsized place `into` autorefs to `&[u32]` (that
// step may always hold), so a `From<&mut [u32]>` impl is not used.
struct My(usize);
impl From<&mut [u32]> for My {
    fn from(s: &mut [u32]) -> My {
        My(s.len())
    }
}
fn main() {
    let mut v = vec![1u32, 2, 3];
    let y: My = v[1..].into();
    println!("{}", y.0);
}
