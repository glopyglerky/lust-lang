// expect: L0202
// rustc E0106: the one parameter has two lifetimes, so elision cannot pick.
fn first(v: &Vec<&str>) -> &str {
    v[0]
}

fn main() {
    let v = vec!["a"];
    println!("{}", first(&v));
}
