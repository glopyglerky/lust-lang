// expect: L0733
// rustc: lifetime may not live long enough. The pointer's output is the type parameter `T`
// (here `&u8` with its own lifetime), not the pointer's input lifetime.
fn take<T>(f: fn(&u8) -> T, x: &u8) -> T {
    f(x)
}
fn main() {
    let y = 2u8;
    let r = take::<&u8>(|a| a, &y);
    println!("{}", r);
}
