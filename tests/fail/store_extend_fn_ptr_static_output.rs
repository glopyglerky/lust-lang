// expect: L0733
// rustc E0521: the array's `fn(&u8) -> &'b u8` items are stored as `fn(&u8)
// -> &'static u8`, so `'b` must be `'static`.
fn k<'b>(c: &mut Vec<fn(&u8) -> &'static u8>, x: fn(&u8) -> &'b u8) {
    let a = [x];
    c.extend(a);
}
fn main() {}
