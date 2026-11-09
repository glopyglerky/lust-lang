// expect: L0735
// rustc: implementation of `Fn` is not general enough. A `let` closure's argument has one inferred lifetime; `dyn Fn(&u8)` takes any.
fn main() {
    let c = |x| println!("{}", x);
    let d: &dyn Fn(&u8) = &c;
    d(&1);
}
