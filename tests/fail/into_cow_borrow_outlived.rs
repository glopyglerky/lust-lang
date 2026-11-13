// expect: L0306
// rustc E0597: the Cow from &String borrows the String.
use std::borrow::Cow;

fn main() {
    let c: Cow<str>;
    {
        let s = String::from("x");
        c = (&s).into();
    }
    println!("{c}");
}
