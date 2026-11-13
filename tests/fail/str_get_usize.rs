// expect: L0612
// `usize` is no `SliceIndex<str>`: `str::get` takes a range only.
fn main() {
    let s = "abc";
    println!("{:?}", s.get(1));
}
