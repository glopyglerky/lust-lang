// ARCH 2.13, A1 form of the ts-rust shape at ast/store.rs:1157
// (`RefCell::new(BuildStores { .. })` in a thread local). The
// real RefCell form is later/pass/std_refcell_static.rs and
// gates with W-STD-B. Here a local cell-like struct with a
// const fn constructor holds a local aggregate in a static.
struct CellLike<T> {
    value: T,
}
impl<T> CellLike<T> {
    const fn new(value: T) -> Self {
        CellLike { value }
    }
}
// Local stand-in for BuildStores: scalars and a fixed array
// stand in for the base offset and the store Vec, which are
// not A1 const material.
struct Stores {
    base: u32,
    slots: [u32; 3],
    len: usize,
}
static BUILD: CellLike<Stores> = CellLike::new(Stores { base: 7, slots: [10, 20, 30], len: 2 });
static EMPTY: CellLike<Stores> = CellLike::new(Stores { base: 0, slots: [0; 3], len: 0 });
fn main() {
    let used = BUILD.value.slots[0] + BUILD.value.slots[BUILD.value.len - 1];
    println!("{} {} {}", BUILD.value.base, used, EMPTY.value.len);
}
