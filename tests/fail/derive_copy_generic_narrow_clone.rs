// expect: L0112
// rustc E0277: `impl<T: Copy> Copy for S<T>` needs `S<T>: Clone`, and the
// written Clone impl covers only S<u8>.
#[derive(Copy)]
struct S<T>(T);
impl Clone for S<u8> {
    fn clone(&self) -> Self {
        S(self.0)
    }
}
fn main() {
    let s = S(1u8);
    println!("{}", s.clone().0);
}
