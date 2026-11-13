// expect: L0102
fn main() {
    let v: std::option::Vec<i32> = Vec::new();
    println!("{}", v.len());
}
