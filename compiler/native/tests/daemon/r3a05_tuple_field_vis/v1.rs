mod m {
    pub struct S(pub i32);
}
fn main() {
    let s = m::S(5);
    println!("{}", s.0);
}
