// expect: L0101
// A `#[test]` fn does not exist in a normal build (rustc E0425).
#[test]
fn helper() -> i32 {
    1
}

fn main() {
    println!("{}", helper());
}
