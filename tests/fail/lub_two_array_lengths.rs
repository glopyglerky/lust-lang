// expect: L0110
// rustc E0308: two array references of different lengths have no LUB
// (neither coerces to the other), even before a slice arm.
fn main() {
    let v = vec![1u8];
    let s = match v.len() {
        0 => &[1u8, 2],
        1 => &[3u8, 4, 5],
        _ => &v[..],
    };
    println!("{}", s.len());
}
