// expect: L0735
// rustc E0599: `filter` passes `&Self::Item`, its own borrow; a `let`
// closure with no annotation fixes that lifetime, so it is not an
// `FnMut(&&String) -> bool` for every lifetime.
fn main() {
    let v = vec![String::from("a")];
    let pred = |x| {
        let y: &&String = x;
        y.len() > 0
    };
    println!("{}", v.iter().filter(pred).count());
}
