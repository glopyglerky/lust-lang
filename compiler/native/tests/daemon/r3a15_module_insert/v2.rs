mod a {
    pub fn v() -> u32 {
        1
    }
}
mod z {
    pub fn v() -> u32 {
        26
    }
}
fn main() {
    println!("{} {}", a::v(), z::v());
}
