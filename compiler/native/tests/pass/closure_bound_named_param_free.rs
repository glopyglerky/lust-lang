// A closure passed for `F: FnMut(T)` takes `T`'s instantiation as its
// argument, so naming the enclosing fn's `'a` there is fine (rustc
// accepts); its argument links to `T`, not to the slice borrow.
fn each<T: Copy, F: FnMut(T)>(xs: &[T], mut f: F) {
    for &x in xs {
        f(x)
    }
}
fn f<'a>(v: &'a [String], keep: &mut Vec<&'a String>) {
    let refs: Vec<&'a String> = v.iter().collect();
    each(&refs, |s: &'a String| keep.push(s));
}
fn main() {
    let v = vec![String::from("a"), String::from("b")];
    let mut k = Vec::new();
    f(&v, &mut k);
    println!("{}", k.len());
}
