// expect: L0735
// rustc E0277: `G: Fn(&u8)` may get a borrow of `inner`'s local, but `F: Fn(&'a u8)` takes only `'a`.
fn inner<G: Fn(&u8)>(g: G) {
    let x = 1u8;
    g(&x)
}

fn outer<'a, F: Fn(&'a u8)>(f: F) {
    inner(f)
}

fn main() {}
