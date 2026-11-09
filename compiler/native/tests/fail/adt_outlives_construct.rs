// expect: L0306
// rustc E0597: `H<'a, 'b: 'a>` makes the `'b` borrow live as long as the `'a` one.
struct H<'a, 'b: 'a> { r: &'a u8, s: &'b u8 }
fn get<'a, 'b>(h: H<'a, 'b>) -> &'a u8 { h.r }
fn main() {
    let v = 3u8;
    let r;
    { let w = 4u8; let h = H { r: &v, s: &w }; r = get(h); }
    println!("{}", r);
}
