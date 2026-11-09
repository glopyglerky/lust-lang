// rustc accepts: a `let` annotation with an explicit lifetime (W-BORROW keeps its regions).
fn main() {
    let s: &'static str = "hi";
    println!("{}", s);
}
