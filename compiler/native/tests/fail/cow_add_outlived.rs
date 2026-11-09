// expect: L0306
// rustc E0597: the sum is a Cow<'a> of the left side's lifetime,
// even though the value it holds is Owned.
use std::borrow::Cow;

fn main() {
    let c2: Cow<str>;
    {
        let s = String::from("ab");
        c2 = Cow::Borrowed(s.as_str()) + "x";
    }
    println!("{}", c2);
}
