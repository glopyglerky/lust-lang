fn main() {
    #[derive(Debug, PartialEq)]
    struct S;
    let x = S;
    match x { S => println!("unit") }
}
