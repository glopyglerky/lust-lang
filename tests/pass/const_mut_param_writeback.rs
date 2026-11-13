// Writes through a `&mut` parameter of a const fn land in the
// caller's local, also through a reborrow passed on to another
// const fn.
const fn inc(x: &mut u32) {
    *x += 1;
}
const fn twice(x: &mut u32) {
    inc(x);
    inc(x);
}
const fn set(x: &mut u32, v: u32) {
    *x = v * 2 + 1;
}
const A: u32 = {
    let mut v: u32 = 1;
    inc(&mut v);
    v
};
const B: u32 = {
    let mut v: u32 = 5;
    twice(&mut v);
    v
};
static C: u32 = {
    let mut v: u32 = 3;
    set(&mut v, 3);
    set(&mut v, 7);
    v
};
fn main() {
    println!("{} {} {}", A, B, C);
}
