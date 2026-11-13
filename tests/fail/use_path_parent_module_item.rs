// expect: L0506
// rustc E0432: a `use` path's first segment reads the current module, not
// its parents: `x` is a root item, so `use x::g` in `a` does not resolve.
mod x {
    pub fn g() -> u32 {
        3
    }
}
mod a {
    use x::g;
    pub fn h() -> u32 {
        g()
    }
}
fn main() {
    println!("{}", a::h());
}
