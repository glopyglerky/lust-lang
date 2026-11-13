// Option::as_deref / as_deref_mut and Result::as_deref / as_deref_mut
// at std's Deref types: String, Vec, Box, &T, &mut T.
fn name_len(o: Option<&str>) -> usize {
    o.map_or(0, |s| s.len())
}

fn main() {
    let s: Option<String> = Some(String::from("hello"));
    let n: Option<String> = None;
    println!("{:?} {:?} {}", s.as_deref(), n.as_deref(), name_len(s.as_deref()));
    let v: Option<Vec<i32>> = Some(vec![1, 2, 3]);
    println!("{:?}", v.as_deref().map(|x| x.len()));
    let mut mv: Option<Vec<i32>> = Some(vec![3, 1, 2]);
    if let Some(x) = mv.as_deref_mut() {
        x[0] = 9;
    }
    println!("{:?}", mv);
    let b: Option<Box<i32>> = Some(Box::new(7));
    println!("{:?}", b.as_deref().copied());
    let mut mb: Option<Box<i32>> = Some(Box::new(1));
    if let Some(x) = mb.as_deref_mut() {
        *x += 10;
    }
    println!("{:?}", mb);
    let k = 4;
    let r: Option<&i32> = Some(&k);
    println!("{:?}", r.as_deref());
    let mut m = 5;
    let mut rm: Option<&mut i32> = Some(&mut m);
    if let Some(x) = rm.as_deref_mut() {
        *x *= 2;
    }
    println!("{:?}", rm.as_deref());
    let res: Result<String, u8> = Ok(String::from("ok"));
    let bad: Result<String, u8> = Err(3);
    println!("{:?} {:?}", res.as_deref(), bad.as_deref());
    let rv: Result<Vec<u8>, String> = Ok(vec![1]);
    println!("{:?}", rv.as_deref().map(|x| x.len()));
    let rb: Result<Box<char>, ()> = Ok(Box::new('z'));
    println!("{:?}", rb.as_deref());
}
