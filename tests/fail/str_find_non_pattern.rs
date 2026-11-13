// expect: L0612
// An integer is no `Pattern` (tf-str).
fn main() {
    println!("{:?}", "abc".find(42));
}
