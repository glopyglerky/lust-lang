// expect: L0203
// rustc E0597: `push` keeps `&local` as `'a`; lust rejects a closure with
// a named parameter that another closure calls.
fn f<'a>(keep: &mut Vec<&'a String>) {
    let mut push = |x: &'a String| keep.push(x);
    let mut call_with_local = || {
        let local = String::new();
        push(&local);
    };
    call_with_local();
}
fn main() {
    let mut v = Vec::new();
    f(&mut v);
    println!("{}", v.len());
}
