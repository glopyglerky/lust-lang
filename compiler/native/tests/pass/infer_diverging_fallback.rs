// A variable a `!` value flows into falls back to `!` (edition 2024),
// so these need no annotation; integer and float literals default.
fn g() -> Result<(), i32> {
    let _ = Err(1)?;
    Ok(())
}
fn h(o: Option<u8>) -> Option<u8> {
    let _ = None?;
    o
}
fn k(b: bool) {
    let _x = Some(return);
    let _ = if b { 1 } else { 2 };
}
fn mk<T>() -> Option<T> {
    None
}
fn n() {
    let _x = match mk() { None => return, Some(v) => v };
    println!("never");
}
fn m() -> u8 {
    let _ = match Some(3u8) { Some(_) => return 4, None => panic!() };
}
fn main() {
    println!("{:?}", g());
    println!("{:?}", h(Some(2)));
    k(true);
    let _ = 1;
    let _ = 2.5;
    Some(3);
    let _ = vec![1, 2].len();
    if false {
        let _ = Some(panic!());
        let _ = Ok::<_, u8>(loop {});
    }
    println!("{}", m());
    n();
}
