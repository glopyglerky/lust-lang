use std::convert::TryFrom;
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
fn from_json(n: usize) -> Result<Vec<String>, String> { Ok((0..n).map(|i| i.to_string()).collect()) }
fn pair(n: usize) -> Result<[String; 2], String> {
    from_json(n)?
        .try_into()
        .map_err(|items: Vec<_>| format!(": expected 2 items, found {}", items.len()))
}
fn generic<T, const N: usize>(v: Vec<T>) -> Option<[T; N]> { v.try_into().ok() }
fn generic_r<T, const N: usize>(v: Vec<T>) -> Result<[T; N], Vec<T>> { v.try_into() }
fn generic2<T: std::fmt::Debug, const N: usize>(v: Vec<T>) -> usize {
    match generic_r::<T, N>(v) { Ok(a) => a.len(), Err(v) => 100 + v.len() }
}
fn main() {
    println!("{:?} {:?}", pair(2), pair(3));
    let v = vec![D(1), D(2)];
    let r: Result<[D; 2], Vec<D>> = v.try_into();
    match r { Ok(a) => println!("ok {}", a[1].0), Err(_) => println!("err") }
    let v = vec![D(3), D(4), D(5)];
    let r: Result<[D; 2], _> = <[D; 2]>::try_from(v);
    match r { Ok(_) => println!("ok"), Err(v) => println!("err {}", v.len()) }
    println!("{:?}", generic::<u8, 3>(vec![1, 2, 3]));
    println!("{:?}", generic::<u8, 2>(vec![1, 2, 3]));
    println!("{}", generic2::<i32, 1>(vec![1]));
    println!("{}", generic2::<i32, 4>(vec![1]));
    let z: Result<[(); 3], Vec<()>> = vec![(), (), ()].try_into();
    println!("{:?}", z);
    let e: Result<[u8; 0], _> = Vec::<u8>::new().try_into();
    println!("{:?}", e);
    let a = <[String; 1] as TryFrom<Vec<String>>>::try_from(vec!["q".to_string()]);
    println!("{:?}", a);
    let w: [u64; 2] = vec![7, 8].try_into().expect("two");
    println!("{:?}", w);
    let mut x = vec![1u16, 2];
    x.push(3);
    let y: [u16; 3] = x.try_into().unwrap_or([0; 3]);
    println!("{:?}", y);
}
