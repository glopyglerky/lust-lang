// expect: L0306
// rustc E0597: the error in the sum's Result still borrows `s`.
fn main() {
    let r: Result<u8, &str>;
    {
        let s = String::from("e");
        let v = vec![Ok(1u8), Err(s.as_str())];
        r = v.into_iter().sum();
    }
    println!("{:?}", r);
}
