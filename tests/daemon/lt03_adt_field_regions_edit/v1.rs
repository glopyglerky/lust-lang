// Swapping a struct's field lifetimes keeps its erased layout; the
// constructing caller's borrows change and must recheck (W-BORROW).
struct P<'a, 'b> {
    x: &'a String,
    y: &'b String,
}

fn take<'a, 'b>(p: P<'a, 'b>, d: &'a String) -> &'a String {
    let _ = p;
    d
}

fn main() {
    let a = String::from("a");
    let r;
    {
        let b = String::from("b");
        r = take(P { x: &a, y: &b }, &a);
    }
    println!("{}", r);
}
