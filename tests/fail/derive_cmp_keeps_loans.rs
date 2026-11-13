// expect: L0306
// rustc E0597: comparing through a derived PartialOrd does not hide
// that the result borrows `b`, which dies first.
#[derive(PartialEq, PartialOrd)]
struct Key<'a> {
    name: &'a String,
}

fn main() {
    let a = String::from("a");
    let k1 = Key { name: &a };
    let r;
    {
        let b = String::from("b");
        let k2 = Key { name: &b };
        r = if k1 < k2 { k1.name } else { k2.name };
    }
    println!("{}", r);
}
