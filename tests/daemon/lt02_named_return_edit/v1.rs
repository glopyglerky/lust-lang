// A named-lifetime return edit (`'a` to `'b`) changes which argument
// the caller's result borrows; the caller must recheck (W-BORROW).
fn pick<'a, 'b>(x: &'a String, y: &'b String) -> &'a String {
    let _ = y;
    x
}

fn main() {
    let a = String::from("a");
    let r;
    {
        let b = String::from("b");
        r = pick(&a, &b);
    }
    println!("{}", r);
}
