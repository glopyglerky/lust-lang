// expect: L0306
// rustc E0597: a borrow through `*b` dies with the Box that owns it.
fn main() {
    let r;
    {
        let b = Box::new(String::from("hi"));
        r = &*b;
    }
    println!("{}", r);
}
