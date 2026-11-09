use std::sync::LazyLock;
fn main() {
    println!("{}", later());
}
fn later() -> u32 { static S: u32 = 6; S }
