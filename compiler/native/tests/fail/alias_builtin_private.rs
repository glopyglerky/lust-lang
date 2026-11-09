// expect: L0510
// rustc E0603: the alias is private to its module.
mod m {
    type V = Vec<u8>;
}
fn main() { let v = m::V::new(); println!("{:?}", v); }
