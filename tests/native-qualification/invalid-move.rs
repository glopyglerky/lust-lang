fn main() {
    let value = String::from("moved");
    let moved = value;
    println!("{} {}", value, moved);
}
