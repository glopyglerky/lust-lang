// expect: L0110
mod m {
    #[derive(Debug, Clone, Copy)]
    pub enum E {
        A,
        B,
    }
}
fn f(e: m::E) -> u8 {
    match e {
        A => 0,
        _ => 1,
    }
}
fn main() {
    println!("{} {}", f(m::E::A), f(m::E::B));
}
