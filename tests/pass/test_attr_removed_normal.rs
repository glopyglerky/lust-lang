// In a normal build a `#[test]` fn is removed before checking, so
// even its test-invalid signature is fine here.
#[test]
fn helper() -> i32 {
    1
}

fn main() {
    println!("hi");
}
