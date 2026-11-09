// capability: rejected: C string literals
// expect: L0641

fn main() {
    let c = c"hi";
    println!("{}", c.to_bytes().len());
}
