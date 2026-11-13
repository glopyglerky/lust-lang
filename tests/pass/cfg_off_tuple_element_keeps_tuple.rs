// A cfg that is off on a tuple element keeps the tuple: `(#[cfg(off)] a, b)`
// is the one-element tuple `(b,)`, also nested. A block-like list element
// that is off runs to its comma.
fn main() {
    let x = 3;
    println!("{:?}", (#[cfg(any())] 1, 2));
    println!("{:?}", ((#[cfg(any())] 1, x), 3));
    println!("{:?}", (1, #[cfg(any())] 2));
    println!("{:?}", (#[cfg(any())] 1,));
    println!("{:?}", vec![#[cfg(any())] match x { _ => 2 }, 10]);
    println!("{:?}", [#[cfg(any())] if x > 0 { 2 } else { 3 }, 10]);
    println!("{:?}", (#[cfg(any())] { 2 }, 10));
}
