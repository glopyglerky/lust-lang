// expect: L0303
fn main() {
    let mut res: Result<String, String> = Ok(String::from("a"));
    let m = res.as_mut();
    let r = res.as_ref();
    println!("{:?} {:?}", m, r);
}
