// expect: L0306
// rustc E0597: the mapped result borrows `entries`, which dies first
// (lust once ran this and read freed memory).
struct Opt<T> { v: Option<T> }
impl<T> Opt<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Opt<U> { Opt { v: match self.v { Some(x) => Some(f(x)), None => None } } }
}
fn main() {
    let m;
    {
        let entries = vec![(String::from("k"), 1u32)];
        let o = Opt { v: entries.iter().find(|e| e.1 == 1) };
        m = o.map(|e| &e.0);
    }
    println!("{:?}", m.v);
}
