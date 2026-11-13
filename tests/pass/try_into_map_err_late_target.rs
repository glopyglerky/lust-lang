// A closure with an annotated parameter takes a `TryInto` error before a
// later use fixes the target: the closure's Fn obligation waits until
// the error type is known.
fn vec2(v: Vec<u8>) -> Result<[u8; 2], String> {
    let a = v.try_into().map_err(|items: Vec<u8>| format!("found {}", items.len()))?;
    Ok(a)
}
fn small(x: u32) -> Result<u8, String> {
    let a = x.try_into().map_err(|e: std::num::TryFromIntError| e.to_string())?;
    Ok(a)
}
fn arr(s: &[u8]) -> Result<[u8; 2], String> {
    let a = s.try_into().map_err(|_e: std::array::TryFromSliceError| "short".to_string())?;
    Ok(a)
}
fn generic<T>(v: Vec<T>) -> Result<[T; 1], usize> {
    let a = v.try_into().map_err(|items: Vec<T>| items.len())?;
    Ok(a)
}
fn main() {
    println!("{:?} {:?}", vec2(vec![1]), vec2(vec![1, 2]));
    println!("{:?} {:?}", small(300), small(7));
    println!("{:?} {:?}", arr(&[1]), arr(&[3, 4]));
    println!("{:?} {:?}", generic(vec!['a']), generic(Vec::<char>::new()));
}
