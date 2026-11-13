// expect: L0311
// rustc E0505: `map_it`'s result borrows `pair` through the closure,
// and `pair` moves first (capture-less closure: no creating statement).
fn map_it<T, U, F: FnOnce(T) -> U>(t: T, f: F) -> U { f(t) }
fn main() {
    let pair = (String::from("a"), String::from("b"));
    let r = map_it(&pair, |p| &p.1);
    drop(pair);
    println!("{}", r);
}
