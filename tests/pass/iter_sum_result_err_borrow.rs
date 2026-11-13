// A sum into `Result<S, E>` holds the items' error only, not the
// borrow of the iterator's source.
fn first_err<'a>(v: &[Result<u32, &'a str>]) -> Result<u32, &'a str> {
    v.iter().cloned().sum()
}
fn main() {
    let r: Result<f64, &str> = [Ok(1.5), Ok(2.0)].iter().cloned().product();
    println!("{:?}", r);
    let s = String::from("e");
    let v = vec![Ok(1), Err(s.as_str())];
    println!("{:?}", first_err(&v));
}
