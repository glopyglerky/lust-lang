// expect: L0733
// rustc E0308: `second` returns its second argument, which `G` passes with any lifetime, not `'b`.
fn second<'x, 'y>(_a: &'y u8, b: &'x u8) -> &'x u8 {
    b
}

fn pick<'b, G: Fn(&'b u8, &u8) -> &'b u8>(x: &'b u8, g: G) -> &'b u8 {
    let l = 2u8;
    g(x, &l)
}

fn main() {
    let k = 1u8;
    println!("{}", pick(&k, second));
}
