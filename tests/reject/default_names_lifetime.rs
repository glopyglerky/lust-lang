// expect: L0614
// A generic parameter default that names a lifetime is not supported yet.
struct Pair<'a, T = &'a str> {
    first: T,
    second: &'a str,
}

fn main() {
    let p: Pair = Pair { first: "a", second: "b" };
    println!("{} {}", p.first, p.second);
}
