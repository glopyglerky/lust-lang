// panic: `at` split index (is 9) should be <= len (is 3)
fn main() {
    let mut v = vec![1, 2, 3];
    println!("before");
    let _t = v.split_off(9);
    println!("unreachable");
}
