// A `let` closure with no annotation and no expected signature takes
// free argument regions (rustc infers them): it may store or return its
// argument, and every call's argument flows into them.
fn apply<T, F: Fn(T) -> usize>(t: T, f: F) -> usize {
    f(t)
}
fn main() {
    let mut v: Vec<&String> = Vec::new();
    let a = String::from("a");
    let b = String::from("bb");
    let mut add = |x| v.push(x);
    add(&a);
    add(&b);
    println!("{}", v.len());
    let id = |x| x;
    let r: &String = id(&a);
    println!("{}", r);
    let len = |x: &String| x.len();
    let words = vec![String::from("abc")];
    let total: usize = words.iter().map(len).fold(0, |acc, n| acc + n);
    let n = apply(&b, |s| s.len());
    let show = |s| println!("{}", s);
    words.iter().for_each(show);
    println!("{} {}", total, n);
}
