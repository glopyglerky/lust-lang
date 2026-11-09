// expect: L0001
// rustc E0658: a `let` initializer takes no outer attribute.
fn main() {
    let v = #[cfg(all())] 5;
    println!("{v}");
}
