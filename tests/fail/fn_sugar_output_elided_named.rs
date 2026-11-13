// expect: L0733
// rustc: lifetime may not live long enough. In `Fn(&'a u8) -> &u8` the elided output lifetime is the only input lifetime, `'a`.
fn app<'a, F: Fn(&'a u8) -> &u8>(x: &'a u8, f: F) -> &'static u8 {
    f(x)
}

fn main() {}
