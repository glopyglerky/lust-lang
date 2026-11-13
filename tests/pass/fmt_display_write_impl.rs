// A user Display impl that writes through write! (was a reject before
// user fmt impls landed).
struct S;

impl std::fmt::Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S")
    }
}

fn main() {
    println!("{}", S);
}
