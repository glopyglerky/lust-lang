fn main() {
    let operation: fn(i32) -> i32 = left::unchecked_add;
    println!("unsafe-to-safe={}", operation(25));
}
