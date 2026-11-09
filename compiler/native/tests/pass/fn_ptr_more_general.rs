// A fn pointer or `dyn Fn` type that takes any lifetime serves where one
// that takes only `&'static` is expected, and a `'static` output serves
// where an argument's lifetime is expected.
static K: u8 = 1;

fn len(s: &str) -> usize {
    s.len()
}

fn st(_: &u8) -> &'static u8 {
    &K
}

fn main() {
    let p: fn(&str) -> usize = len;
    let q: fn(&'static str) -> usize = p;
    println!("{}", q("ab"));
    let c = |s: &str| s.len();
    let d: &dyn Fn(&str) -> usize = &c;
    let e: &dyn Fn(&'static str) -> usize = d;
    println!("{}", e("abc"));
    let r: fn(&u8) -> &'static u8 = st;
    let t: fn(&u8) -> &u8 = r;
    let l = 2u8;
    println!("{}", t(&l));
}
