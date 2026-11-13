// A closure under `&` becomes a fn pointer only through a coercion site
// inside the borrow: a block, an array, a tuple, an `if`, a call arg.
fn main() {
    let a: &fn(u8) -> u8 = &{ |a| a };
    let b: &[fn(u8) -> u8; 1] = &[|a| a];
    let c: &(fn(u8) -> u8,) = &(|a| a,);
    let d: &Option<fn(u8) -> u8> = &Some(|a| a);
    let e: &fn(u8) -> u8 = &if true { |a| a } else { |a| a + 1 };
    println!("{} {} {} {} {}", a(1), b[0](2), (c.0)(3), d.unwrap()(4), e(5));
}
