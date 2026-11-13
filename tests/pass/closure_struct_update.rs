// Struct update syntax in a closure captures the unnamed fields.
#[derive(Debug)]
struct Opts {
    a: u8,
    b: u8,
    name: String,
}
fn main() {
    let base = Opts { a: 1, b: 2, name: String::from("base") };
    let make = move || Opts { a: 9, ..base };
    println!("{:?}", make());
    let o = Opts { a: 3, b: 4, name: String::from("o") };
    let bump = || Opts { name: String::from("new"), ..o };
    println!("{:?} {}", bump(), o.name);
}
