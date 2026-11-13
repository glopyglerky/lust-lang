// A user fn named `size_of_val` shadows the prelude one (Rust 1.80+).
fn size_of_val(_x: &u8) -> usize {
    42
}

fn main() {
    println!("{} {}", size_of_val(&1u8), std::mem::size_of_val(&1u16));
}
