// expect: L0306
// rustc E0597: Option<&T> from &T keeps the borrow.
fn main() {
    let r: Option<&String>;
    {
        let s = String::new();
        r = (&s).into();
    }
    println!("{r:?}");
}
