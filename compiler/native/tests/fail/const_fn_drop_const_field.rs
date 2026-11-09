// expect: L0631
// rustc E0493: `C.1` in a const fn drops its copy of `C`, even when the fn is never called.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
const C: (D, u32) = (D(1), 5);
const fn f() -> u32 { C.1 }
fn main() { println!("ok"); let _ = f; }
