// expect: L0110
// rustc E0308: an impl's lifetime is early-bound for its methods, so the
// method does not coerce to a fn pointer generic over that lifetime.
struct W<'a>(&'a str);

impl<'a> W<'a> {
    fn get(x: &'a str) -> &'a str {
        x
    }
}

fn main() {
    let f: fn(&str) -> &str = W::get;
    println!("{}", f("a"));
}
