// A region-only edit of a lifetime alias's body (`type P<'a, 'b>`): the
// unchanged `first` and `main` must recheck (W-BORROW, driver alias
// fingerprint).
type P<'a, 'b> = (&'b String, &'a String);

fn first<'a, 'b>(p: P<'a, 'b>) -> &'a String {
    p.0
}

fn main() {
    let a = String::from("a");
    let r;
    {
        let b = String::from("b");
        r = first((&a, &b));
    }
    println!("{}", r);
}
