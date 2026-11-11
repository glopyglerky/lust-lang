fn main() {
    let value = unsafe {
        let first = { left::unchecked_add(8) };
        { left::unchecked_add(first) }
    };
    println!("unsafe-nested={}", value);
}
