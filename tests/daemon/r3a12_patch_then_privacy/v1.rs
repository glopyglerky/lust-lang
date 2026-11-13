mod a {
    pub fn f() -> u32 {
        1
    }
}
mod b {
    pub fn f() -> u32 {
        2
    }
}
fn main() {
    println!("{}", a::f());
}
