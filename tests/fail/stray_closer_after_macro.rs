// expect: L0001
// A stray closer is the first error, before a builtin macro call above it.
fn main() {
    println!("{}", column!());
}
fn broken() { ) }
