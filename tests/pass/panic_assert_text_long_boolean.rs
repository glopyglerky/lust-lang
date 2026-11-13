// Long binary expressions use the pinned Rust pretty-printer's breaks.
// panic: assertion failed:
fn main() {
    let (first_value, second_value, third_value) = (1, 2, 3);
    assert!(first_value == 1 && second_value == 2 && third_value == 3 && first_value + second_value == 9);
}
