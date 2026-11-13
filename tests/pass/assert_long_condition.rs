// A condition wider than the pretty-printer margin still compiles.
fn main() {
    let (first_value, second_value, third_value) = (1, 2, 3);
    assert!(first_value == 1 && second_value == 2 && third_value == 3 && first_value + second_value == 3);
    println!("ok");
}
