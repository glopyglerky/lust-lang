// expect: L0735
// rustc: "implementation of `FnMut` is not general enough". `y` has no
// annotation, so its lifetime is fixed; the bound passes `apply`'s own
// borrow there.
fn apply<F: FnMut(&String, &String) -> usize>(mut f: F) -> usize {
    let a = String::from("a");
    f(&a, &a)
}
fn main() {
    let mut n = 0;
    let f = |x: &String, y| {
        let _z: &String = y;
        n += 1;
        x.len()
    };
    println!("{}", apply(f));
    println!("{}", n);
}
