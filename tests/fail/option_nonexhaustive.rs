// expect: L0115
fn main() {
    let x: Option<i32> = None;
    match x {
        Some(v) => println!("{}", v),
    }
}
