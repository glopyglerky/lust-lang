// A user `map` with `F: FnOnce(T) -> U` (the shape of a source-std
// `Option::map`): the result borrows the found entry.
struct Opt<T> { v: Option<T> }
impl<T> Opt<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Opt<U> { Opt { v: match self.v { Some(x) => Some(f(x)), None => None } } }
}
fn main() {
    let entries = vec![(String::from("k"), 1u32)];
    let o = Opt { v: entries.iter().find(|e| e.1 == 1) };
    let m = o.map(|e| &e.0);
    println!("{:?}", m.v);
}
