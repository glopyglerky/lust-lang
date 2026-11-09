// `F: FnMut(T)` with `T = &String`: the closure keeps the longest item.
fn each<T: Copy, F: FnMut(T)>(xs: &[T], mut f: F) { for &x in xs { f(x) } }
fn main() {
    let a = String::from("a");
    let b = String::from("b");
    let refs = vec![&a, &b];
    let mut longest: &String = &a;
    each(&refs, |s| if s.len() >= longest.len() { longest = s; });
    println!("{}", longest);
}
