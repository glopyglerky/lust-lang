// expect: L0110
// rustc E0521: the array element keeps the higher-ranked type of `x`, so
// `extend` meets `for<'r> fn(&'r u8)` where `fn(&'b u8)` is wanted (a
// direct `push(x)` coerces and is fine).
fn k<'b>(v: &mut Vec<&'b fn(&'b u8)>, x: &'b fn(&u8)) {
    v.extend([x]);
}
fn main() {}
