// A `&'static` written in a fn pointer type: the call passes a `'static`
// borrow, and a fn that takes any lifetime coerces to the pointer.
fn apply(f: fn(&'static str) -> usize) -> usize {
    f("abc")
}

fn len(s: &str) -> usize {
    s.len()
}

fn main() {
    println!("{}", apply(len));
}
