// A by-value match arm in a closure moves only `p.a`: the closure
// captures just that field, and `p.b` stays usable.
struct P { a: String, b: String }
fn main() {
    let p = P { a: String::from("a"), b: String::from("b") };
    let first = || match p { P { a, .. } => a.len() };
    println!("{} {}", first(), p.b);
}
