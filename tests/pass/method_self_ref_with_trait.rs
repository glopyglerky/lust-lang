trait Marker {}
struct S {
    n: u32,
}
impl S {
    fn a(&self) -> u32 {
        self.n
    }
    fn b(&self) -> u32 {
        self.a() + 1
    }
}
fn main() {
    let s = S { n: 41 };
    println!("{}", s.b());
}
