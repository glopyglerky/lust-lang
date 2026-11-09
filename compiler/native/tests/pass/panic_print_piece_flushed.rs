// Rust's print! writes each piece to stdout's LineWriter before the next
// argument formats: "line1\n" is out when the Display impl panics.
// panic: in fmt
struct S;
impl std::fmt::Display for S {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        panic!("in fmt")
    }
}
fn main() {
    println!("line1\n{}", S);
}
