// A closure passed to `F: FnOnce(T) -> U` returns a borrow of its argument.
fn map_it<T, U, F: FnOnce(T) -> U>(t: T, f: F) -> U { f(t) }
fn main() {
    let pair = (String::from("a"), String::from("b"));
    let r = map_it(&pair, |p| &p.1);
    println!("{}", r);
}
