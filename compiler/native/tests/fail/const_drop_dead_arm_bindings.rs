// expect: L0631
// rustc E0493: a dead arm that uses its bindings still drops its local.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
enum E {
    A(u32),
    B(u32, u32),
}
const X: u32 = match E::A(1) {
    E::A(n) => n,
    E::B(a, b) => {
        let _d = D(a + b);
        0
    }
};
fn main() { println!("{}", X); }
