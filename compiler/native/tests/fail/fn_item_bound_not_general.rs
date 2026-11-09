// expect: L0735
// rustc: implementation of `Fn` is not general enough. `same` needs both arguments to share one lifetime, but `G` gives the second any lifetime.
fn same<'x>(a: &'x u8, _b: &'x u8) -> &'x u8 {
    a
}

fn pick<'b, G: Fn(&'b u8, &u8) -> &'b u8>(x: &'b u8, g: G) -> &'b u8 {
    let l = 2u8;
    g(x, &l)
}

fn main() {
    let k = 1u8;
    println!("{}", pick(&k, same));
}
