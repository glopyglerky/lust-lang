fn main() {
    let text = no_std_explicit_alloc::owned();
    let values = no_std_explicit_alloc::values();
    assert_eq!(text, "core and alloc");
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], 10);
    assert_eq!(values[1], 20);
    assert_eq!(values[2], 12);
    println!("{} {:?}", text, values);
}
