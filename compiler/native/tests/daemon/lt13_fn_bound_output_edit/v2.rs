// Region-only edits of `Fn` bound outputs (W-BORROW): v2 makes `inner`'s
// output `'static`, so the unchanged `outer`, whose `F` returns `'a`,
// must recheck and fail (`entail_bound`); v3 makes `outer`'s `F` return
// `'static` too, so the unchanged `main`, whose closure returns a borrow
// of a local, must recheck and fail (`tie_bound_output`).
fn inner<'b, G: Fn() -> &'static u8>(g: G) -> &'b u8 {
    g()
}
fn outer<'a, F: Fn() -> &'a u8>(f: F) -> &'a u8 {
    inner(f)
}
fn main() {
    let k = 3u8;
    println!("{}", outer(|| &k));
}
