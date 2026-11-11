fn main() {
    let first = unsafe { left::unchecked_add(8) };
    println!("unsafe-leak={}", left::unchecked_add(first));
}
