// A str literal and a promoted constant keep one address however their fn is reached: a
// release C unit that holds a copy of the fn (LUST_PGO_COPIES) has its own kconst slots,
// which must still give the canonical addresses, as rustc gives one allocation.
fn lit() -> &'static str {
    "copied literal"
}

fn arr() -> &'static [u32; 3] {
    &[7, 8, 9]
}

fn pair() -> &'static (u8, &'static str) {
    &(1, "inner")
}

fn via_a() -> (*const u8, *const [u32; 3], *const u8) {
    (lit().as_ptr(), arr(), pair().1.as_ptr())
}

fn via_b(k: u32) -> (*const u8, *const [u32; 3], *const u8) {
    let mut out = (lit().as_ptr(), arr() as *const [u32; 3], pair().1.as_ptr());
    for _ in 0..k {
        out = (lit().as_ptr(), arr(), pair().1.as_ptr());
    }
    out
}

fn main() {
    let a = via_a();
    let b = via_b(3);
    println!("{} {} {}", a.0 == b.0, a.1 == b.1, a.2 == b.2);
    println!("{} {} {}", lit().as_ptr() == a.0, std::ptr::eq(arr(), a.1), pair().1.as_ptr() == b.2);
    println!("{} {:?} {:?}", lit(), arr(), pair());
}
