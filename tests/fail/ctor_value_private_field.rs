// expect: L0510
// rustc E0603: naming a tuple struct constructor with a private field as a
// value is an error, called or not.
mod m {
    #[derive(Debug)]
    pub struct P(u8);
}
fn main() {
    println!("{:?}", Some(3).map(m::P));
}
