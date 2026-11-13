// `Option::map` takes `F: FnOnce`: the shim runs `FnOnce::call_once`,
// core/src/ops/function.rs:250:5 (not a line of std's own source).
fn main() {
    let a: Option<u32> = Some(64);
    println!("{:?}", a.map(u32::ilog2));
    let b: Result<i32, ()> = Ok(49);
    println!("{:?}", b.map(i32::isqrt));
    let c = [3u32, 1].into_iter().fold(7, &u32::div_euclid);
    println!("{}", c);
    let d: Option<i32> = Some(-1);
    println!("{:?}", d.map(i32::isqrt));
}
