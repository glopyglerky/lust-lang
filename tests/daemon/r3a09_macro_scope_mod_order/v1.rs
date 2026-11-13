macro_rules! m {
    () => {
        1
    };
}
mod a { pub fn v() -> u32 { m!() } }
macro_rules! m {
    () => {
        2
    };
}
mod b { pub fn v() -> u32 { m!() } }
fn main() {
    println!("{} {}", a::v(), b::v());
}
