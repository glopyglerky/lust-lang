// A fn pointer type in a top-level tuple struct field is not a fn
// item: the driver's token scan took the next item's braces as its
// body ("internal: function count mismatch").
struct Via(fn(u8) -> i32);
struct Other {
    x: u8,
}
fn twice(x: u8) -> i32 {
    x as i32 * 2
}
fn main() {
    let v = Via(twice);
    let o = Other { x: 3 };
    println!("{} {}", (v.0)(o.x), o.x);
}
