// expect: L0610
// rustc E0282: matching on `None` does not fix its type.
fn main() {
    match None {
        Some(_) => println!("some"),
        None => println!("none"),
    }
}
