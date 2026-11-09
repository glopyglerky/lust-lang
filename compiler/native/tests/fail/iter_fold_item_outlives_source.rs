// expect: L0306
// rustc E0597: the closure returns the item itself, a borrow of `v`, so
// `fold`'s result borrows `v` past its scope.
fn main() {
    let r: &String;
    {
        let v = vec![String::from("a"), String::from("bb")];
        r = v.iter().fold(&v[0], |a, x| if x.len() > a.len() { x } else { a });
    }
    println!("{}", r);
}
