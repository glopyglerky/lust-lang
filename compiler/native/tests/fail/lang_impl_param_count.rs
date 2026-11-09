// expect: L0110
// rustc E0050: `default` takes no parameters.
#[derive(Debug)]
struct S(u8);
impl Default for S {
    fn default(x: u8) -> S {
        S(x)
    }
}
fn main() {
    println!("{:?}", S(1));
}
