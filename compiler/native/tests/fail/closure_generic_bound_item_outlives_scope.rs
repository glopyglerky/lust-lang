// expect: L0306
// rustc E0597: the closure stores a borrow of `b` (through `refs`) in
// `longest`, which outlives them.
fn each<T: Copy, F: FnMut(T)>(xs: &[T], mut f: F) { for &x in xs { f(x) } }
fn main() {
    let a = String::from("a");
    let mut longest: &String = &a;
    {
        let b = String::from("bbb");
        let refs = vec![&a, &b];
        each(&refs, |s| if s.len() >= longest.len() { longest = s; });
    }
    println!("{}", longest);
}
