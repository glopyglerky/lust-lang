// `{:p}` of references, raw pointers, and fn pointers (addresses vary, so
// only their shape and the null pointer print).
fn h() {}
fn main() {
    let x = 7u64;
    let r = &x;
    let p: *const u64 = r;
    println!("{} {}", format!("{:p}", r) == format!("{:p}", p), format!("{:p}", r).starts_with("0x"));
    println!("{} {}", format!("{:#p}", r).len(), format!("{:>30p}|", r).len());
    let f: fn() = h;
    println!("{}", format!("{:p}", f).starts_with("0x"));
    let q: *const u8 = std::ptr::null();
    println!("{:p} {:#p} {:8p}| {:<6p}|", q, q, q, q);
}
