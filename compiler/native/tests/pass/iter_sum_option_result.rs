// sum and product into Option and Result (core's try_process): the first
// None or Err stops the pull and is the result.
fn main() {
    let v = vec![Some(1), Some(2)];
    let s: Option<i32> = v.into_iter().sum();
    println!("{:?}", s);
    let mut seen = 0;
    let s: Option<i32> = vec![Some(1), None, Some(3)].into_iter().inspect(|_| seen += 1).sum();
    println!("{:?} {seen}", s);
    let p: Option<u64> = (1..5).map(Some).product();
    println!("{:?}", p);
    let r: Result<i32, std::num::ParseIntError> = ["1", "2", "x"].iter().map(|x| x.parse::<i32>()).sum();
    println!("{}", r.is_err());
    let r: Result<u8, String> = vec![Ok(1), Ok(3)].into_iter().sum();
    println!("{:?}", r);
    let r: Result<u8, String> = vec![Ok(1), Err("bad".to_string()), Ok(3)].into_iter().sum();
    println!("{:?}", r);
    let f: Option<f64> = Vec::<Option<f64>>::new().into_iter().sum();
    println!("{:?}", f);
    let refs: Option<i32> = [1, 2, 3].iter().map(Some).sum();
    println!("{:?}", refs);
}
