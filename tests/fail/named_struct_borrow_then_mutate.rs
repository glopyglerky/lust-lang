// expect: L0303
// rustc E0502: `f` holds a shared borrow of `v` while `v` is mutated.
struct Finder<'n> {
    needle: &'n Vec<u8>,
}

fn main() {
    let mut v = vec![1u8];
    let f = Finder { needle: &v };
    v.push(2);
    println!("{}", f.needle.len());
}
