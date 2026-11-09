// expect: L0306
// rustc E0716: a closure that captures `k` is not promoted, so its temporary ends with the statement.
fn main() {
    let r: &dyn Fn(i32) -> i32;
    {
        let k = 3;
        r = &|n| n + k;
    }
    println!("{}", r(1));
}
