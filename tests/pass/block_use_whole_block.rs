// A block's `use` and items cover the whole block, also the code before
// them, and a block `use std::...` beats a crate-root item of the name.
fn w() -> i32 { 1 }
fn swap(a: &mut i32, b: &mut i32) { *a = 0; *b = 0; }
mod m {
    pub fn w() -> i32 { 2 }
    pub const C: i32 = 30;
}
fn main() {
    let x = w();
    let y = { k() + C };
    use m::w;
    use m::C;
    fn k() -> i32 { 4 }
    println!("{} {}", x, y);
    let (mut a, mut b) = (1, 2);
    {
        swap(&mut a, &mut b);
        use std::mem::swap;
    }
    println!("{} {}", a, b);
}
