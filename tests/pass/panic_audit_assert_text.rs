// `assert!` with no message prints the condition as rustc's pretty
// printer writes it: operators spaced, calls and macros as written.
// panic: assertion failed: v.len() > 2 && matches!(v.first(), Some(&9)) || (k == 9)
fn main() {
    let k = std::env::args().count() as i32;
    let v = vec![k, 2, 3];
    assert!(v.len()>2&&v[0]==1, "first {}", "check");
    assert!(!v.is_empty());
    println!("ok");
    assert!(v.len()>2&&matches!(v.first(), Some(&9))||(k==9));
}
