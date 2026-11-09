// A coercion cast is checked at body end after literal fallback: a later
// use may still fix a literal's type, and the cast may fix a type var
// nothing else fixes.
fn main() {
    let c = || 1;
    let b = &c as &dyn Fn() -> u8;
    let r: u8 = c();
    let s = &[1u8, 2] as &[u8];
    let t = &[3, 4] as &[i32];
    let v = Vec::new();
    let w = &v as &[u8];
    println!("{} {} {:?} {:?} {:?}", b(), r, s, t, w);
}
