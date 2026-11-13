// expect: L0104
// rustc E0599: an `as _` trait import counts only in its own module.
mod m {
    use std::fmt::Write as _;

    pub fn f(s: &mut String) {
        let _ = write!(s, "x");
    }
}

fn main() {
    let mut s = String::new();
    m::f(&mut s);
    let _ = write!(s, "y");
    println!("{s}");
}
