// expect: L0303
fn main() {
    let mut a = [1, 2, 3];
    let s = &a[..];
    a[0] = 9;
    println!("{}", s[0]);
}
