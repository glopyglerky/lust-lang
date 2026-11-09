// expect: L0612
// rustc E0277: the bound `S: Into<String>` does not hold for u8.
fn f<S: Into<String>>(s: S) -> String {
    s.into()
}

fn main() {
    println!("{}", f(5u8));
}
