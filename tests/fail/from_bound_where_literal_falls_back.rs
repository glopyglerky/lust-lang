// expect: L0612
// rustc E0277: `where u64: From<T>` does not make the literal a u64; it is i32.
fn f<T>(t: T) -> u64
where
    u64: From<T>,
{
    u64::from(t)
}
fn main() {
    let x = 300;
    println!("{}", f(x));
}
