// A `'static`-returning closure may read its captures without returning them.
fn main() {
    let n = 5;
    let f = || -> &'static str { if n > 3 { "a" } else { "b" } };
    println!("{}", f());
}
