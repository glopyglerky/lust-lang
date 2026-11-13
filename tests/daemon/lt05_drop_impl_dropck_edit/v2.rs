// Adding a `Drop` impl makes the struct's borrow live at its drop
// (drop check); the unchanged `main` must recheck (W-BORROW).
struct S<'a> {
    r: &'a String,
}

fn main() {
    let s;
    let x = String::from("x");
    s = S { r: &x };
    println!("{}", s.r);
}

impl Drop for S<'_> {
    fn drop(&mut self) {}
}
