// expect: L0612
// Valid Rust. std's `repeat_n` calls Clone, but lust copies a Copy element
// bitwise, so a Copy type with a written Clone impl is rejected.
#[derive(Copy)]
struct P;
impl Clone for P {
    fn clone(&self) -> P {
        println!("clone");
        *self
    }
}

fn main() {
    let v: Vec<P> = std::iter::repeat_n(P, 3).collect();
    println!("{}", v.len());
}
