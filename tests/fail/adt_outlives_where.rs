// expect: L0306
// rustc E0597: a struct's `where 'b: 'a` holds at every use of the type.
struct H<'a, 'b> where 'b: 'a { r: &'a u8, s: &'b u8 }
fn get<'a, 'b>(h: H<'a, 'b>) -> &'a u8 { h.r }
fn main() {
    let v = 3u8;
    let r;
    { let w = 4u8; let h = H { r: &v, s: &w }; r = get(h); }
    println!("{}", r);
}
