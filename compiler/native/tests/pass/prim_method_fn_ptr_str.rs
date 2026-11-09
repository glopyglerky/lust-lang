// A `str` builtin method that cannot panic as a fn pointer (prints 2): its
// codegen fn has no panic site to match.
fn main() {
    let f: fn(&str) -> usize = str::len;
    println!("{}", f("ab"));
}
