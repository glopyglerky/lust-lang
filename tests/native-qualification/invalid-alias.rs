fn main() {
    let mut value = 7;
    let first = &mut value;
    let second = &mut value;
    println!("{} {}", first, second);
}
