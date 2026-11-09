#[path = "shared.rs"]
mod first;

#[path = "shared.rs"]
mod second;

fn main() {
    println!("shared {}", first::v() + second::v());
}
