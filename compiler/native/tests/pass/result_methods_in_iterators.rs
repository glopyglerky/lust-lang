// Result methods inside iterator closures.
fn main() {
    let v: Vec<Result<i32, String>> = vec![Ok(1), Err(String::from("x")), Ok(3)];
    let oks: Vec<i32> = v.iter().filter(|r| r.is_ok()).map(|r| r.clone().unwrap()).collect();
    let errs: Vec<String> = v.into_iter().filter_map(|r| r.err()).collect();
    println!("{:?} {:?}", oks, errs);
}
