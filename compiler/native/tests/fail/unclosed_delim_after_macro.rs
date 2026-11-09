// expect: L0001
// An unclosed delimiter is the first error, before a builtin macro call above it.
fn main() {
    println!("{}", line!());
}
fn broken( {
