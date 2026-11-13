// expect: L0110
// rustc E0308: an integer for a bool const param in a type.
struct C<const N: bool>;
fn f(_c: C<3>) {}
fn main() {
    println!("ok");
}
