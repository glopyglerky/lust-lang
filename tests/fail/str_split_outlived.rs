// expect: L0306
// rustc E0597: a split item borrows its haystack.
fn main() {
    let first;
    {
        let s = String::from("a,b");
        first = s.split(',').next();
    }
    println!("{first:?}");
}
