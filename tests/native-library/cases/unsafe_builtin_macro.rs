fn main() {
    unsafe {
        assert_eq!(left::unchecked_add(25), 42);
    }
    println!("unsafe-macro=42");
}
