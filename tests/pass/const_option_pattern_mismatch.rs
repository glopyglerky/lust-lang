// `Some(..)` against a `None` const value (and back) does not match in
// const evaluation; the next arm runs.
const N: Option<u32> = None;
const X: u32 = if let Some(n) = N { n } else { 7 };
const Y: u32 = match Some(4u32) {
    None => 1,
    Some(n) => n,
};
const Z: u8 = match Some(Some(3u8)) {
    Some(None) => 1,
    Some(Some(n)) => n,
    None => 0,
};
fn main() {
    println!("{} {} {}", X, Y, Z);
}
