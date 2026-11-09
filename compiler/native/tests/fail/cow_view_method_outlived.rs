// expect: L0306
// rustc E0597: a str method's result through the Cow's view borrows the Cow.
use std::borrow::Cow;

fn main() {
    let r: &str;
    {
        let c: Cow<str> = Cow::Owned(String::from(" x"));
        r = c.trim();
    }
    println!("{r}");
}
