// expect: L0306
// rustc E0597: a `'static` closure parameter takes a local's borrow.

fn main() {
    let show = |x: &'static str| x.len();
    let local = String::from("l");
    println!("{}", show(&local));
}
