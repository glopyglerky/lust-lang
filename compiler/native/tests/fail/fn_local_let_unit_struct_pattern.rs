// expect: L0110
// `let S = 5u8;` under a block `struct S;` is a path pattern of type S,
// so the initializer's type does not match (rustc E0308).
fn main() {
    #[derive(Debug)]
    struct S;
    let S = 5u8;
    println!("{:?}", S);
}
