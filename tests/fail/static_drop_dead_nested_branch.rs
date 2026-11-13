// expect: L0631
// rustc E0493: a dead branch inside a dead arm still drops.
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {}
}
static X: u32 = match 3u32 {
    3 => 1,
    n => {
        if n > 100 {
            let _d = D(n);
        }
        0
    }
};
fn main() { println!("{}", X); }
