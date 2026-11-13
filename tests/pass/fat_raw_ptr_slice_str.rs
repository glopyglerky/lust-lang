// Raw pointers to `[T]` and `str` (tf-fatptr): made by coercion, `as`,
// `ptr::from_ref`/`from_mut`; read by `len`, `cast`, `addr`, `is_null`,
// `==` and `ptr::eq` (address and length), `ptr::addr_eq` (address).
fn len_of(p: *const [u8]) -> usize {
    p.len()
}
fn mk(s: &str) -> *const str {
    s
}
fn main() {
    let v = vec![1u8, 2, 3];
    let sp: *const [u8] = &v[..];
    let sp2: *const [u8] = &v[1..];
    println!("{} {} {}", sp.len(), sp2.len(), sp.cast::<u8>() == v.as_ptr());
    println!("{} {} {} {}", sp == sp, sp != sp2, std::ptr::eq(sp, sp2), std::ptr::eq(sp, &v[..]));
    println!("{} {}", std::ptr::addr_eq(sp, v.as_ptr()), std::ptr::addr_eq(&v[..1] as *const [u8], sp));
    let s = "hello";
    let p: *const str = s;
    let q = std::ptr::from_ref(s);
    let r = std::ptr::from_ref(&s[0..2]);
    println!("{} {} {} {}", p == q, std::ptr::eq(p, q), std::ptr::eq(p, r), std::ptr::addr_eq(p, r));
    println!("{} {}", p.cast::<u8>() == s.as_ptr(), mk("abc").cast::<u8>().is_null());
    let w = [1u16, 2, 3];
    let wp: *const [u16] = &w[..];
    let t = wp as *const u16;
    let u = wp as *const u8 as usize;
    let m = wp as *mut [u16];
    let back: *const [u16] = m;
    println!("{} {} {} {}", t == w.as_ptr(), u == w.as_ptr() as usize, back == wp, m.cast_const() == wp);
    let mut a = [1u8, 2];
    let mp: *mut [u8] = std::ptr::from_mut(&mut a[..]);
    let c: *const [u8] = mp;
    println!("{} {} {}", c.len(), mp.len(), len_of(&[1, 2, 3, 4][..]));
    let arr = [1u32, 2, 3];
    let ap: *const [u32; 3] = &arr;
    let up: *const [u32] = ap;
    let up2: *const [u32] = &arr;
    let up3 = ap as *const [u32];
    let mut z = [0u8; 2];
    let zp: *mut [u8] = &mut z;
    println!("{} {} {} {} {}", up.len(), up2.len(), up3.len(), zp.len(), up.cast::<u32>() == ap.cast::<u32>());
    println!("{} {} {}", std::mem::size_of::<*const [u8]>(), std::mem::size_of::<Option<*const str>>(), std::mem::align_of::<*mut [u64]>());
}
