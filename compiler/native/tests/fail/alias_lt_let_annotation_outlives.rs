// expect: L0306
// rustc E0597: `r: R<'_>` holds `&s` and is used after `s` dies.

type R<'a> = (&'a String, u32);
fn main() {
    let r: R<'_>;
    {
        let s = String::from("x");
        r = (&s, 1);
    }
    println!("{}", r.1);
}
