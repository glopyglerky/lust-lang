// `let r: R<'_>` with a lifetime alias; the borrow ends before `s` dies.
type R<'a> = (&'a String, u32);
fn main() {
    let r: R<'_>;
    {
        let s = String::from("x");
        r = (&s, 1);
        println!("{}", r.0);
    }
}
