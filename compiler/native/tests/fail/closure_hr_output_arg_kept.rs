// expect: L0306
// rustc E0597: the closure returns a borrow of its argument, so the call's result borrows `v`, which ends first.
fn first<T, F: Fn(&[T]) -> &T>(v: &[T], f: F) -> &T {
    f(v)
}

fn main() {
    let r;
    {
        let v = vec![String::from("p")];
        r = first(&v, |s| &s[0]);
    }
    println!("{}", r);
}
