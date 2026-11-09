// expect: L0306
// rustc E0597: the same through a direct `next` on the iterator.
fn main() {
    let mut v: Vec<&str> = vec!["a"];
    {
        let s = String::from("x");
        let mut it = v.iter_mut();
        if let Some(r) = it.next() {
            *r = &s;
        }
    }
    println!("{:?}", v);
}
