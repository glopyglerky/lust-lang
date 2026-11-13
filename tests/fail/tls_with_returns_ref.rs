// expect: L0733
// The closure's reference cannot leave `with` (rustc: lifetime may not
// live long enough): the closure argument is higher-ranked.
thread_local! {
    static S: Vec<i32> = Vec::new();
}
fn main() {
    let r: &Vec<i32> = S.with(|v| v);
    println!("{}", r.len());
}
