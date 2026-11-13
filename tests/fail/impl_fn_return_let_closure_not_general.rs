// expect: L0735
// rustc: implementation of `Fn` is not general enough. A `let` closure's
// unannotated param takes one inferred lifetime.
fn mk() -> impl Fn(&u8) -> &u8 {
    let f = |a| a;
    f
}
fn main() {
    let f = mk();
    let k = 2u8;
    println!("{}", f(&k));
}
