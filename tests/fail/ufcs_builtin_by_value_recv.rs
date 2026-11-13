// expect: L0110
// rustc E0308: `Vec::len` takes `&Vec<T>`; a path call does not autoref.
fn main() {
    let v = vec![1, 2];
    println!("{}", Vec::len(v));
}
