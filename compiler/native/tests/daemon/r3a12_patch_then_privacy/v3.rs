mod a {
    pub fn f() -> u32 {
        1
    }
}
mod b {
    fn f() -> u32 {
        2
    }
}
fn main() {
    println!("{}", b::f());
}
