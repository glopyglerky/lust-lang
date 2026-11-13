pub const K: usize = 3;

pub struct P<const N: usize>;

pub fn f(x: [u8; 1 - 2]) -> u32 {
    let _ = &x;
    7
}

pub fn g(y: u32) -> u32 {
    y + 1
}

fn main() {
    println!("{} {}", f([]), g(K as u32));
}
