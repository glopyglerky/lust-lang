// expect: L0735
// rustc E0277: `F: Fn(&'a u8)` does not take any lifetime, so it cannot become `dyn Fn(&u8)`.
fn coerce<'a, F: Fn(&'a u8)>(g: &F) {
    let d: &dyn Fn(&u8) = g;
    let l = 1u8;
    d(&l);
}

fn main() {}
