// W-BORROW3: fn items with different lifetimes join where rustc joins
// them: `if`/`else` with and without a fn pointer annotation, arrays of
// fns with the same lifetimes, a `'static` element type, and arrays whose
// later fns are subtypes of the first.
static S: u8 = 4;
fn f_el(a: &u8, b: &u8) -> u8 { *a + *b }
fn f_el2(a: &u8, b: &u8) -> u8 { *a * 10 + *b }
fn f_sh<'x>(a: &'x u8, b: &'x u8) -> u8 { *a * *b }
fn a_st(x: &'static u8) -> u8 { *x + 1 }
fn g_id(x: &u8) -> Option<&u8> { Some(x) }
fn g_st(_x: &u8) -> Option<&'static u8> { Some(&S) }
fn a_st2(x: &'static u8) -> u8 { *x + 2 }
fn main() {
    let c = std::env::args().count() > 5;
    let v = if c { f_el } else { f_sh };
    let w: fn(&u8, &u8) -> u8 = if c { f_sh } else { f_el };
    let arr = [f_el, f_el2];
    // Without an element type, a later fn joins when it is a subtype of
    // the first (rustc coerces it to the first's pointer type).
    let sub = [f_el, f_sh];
    let sub2 = [f_sh, f_el];
    let opt = [g_id, g_st];
    let st: [fn(&'static u8) -> u8; 2] = [a_st, a_st2];
    let (x, y) = (2u8, 3u8);
    println!("{} {} {} {} {}", v(&x, &y), w(&x, &y), arr[1](&x, &y), st[0](&S), st[1](&S));
    println!("{} {} {:?} {:?}", sub[1](&x, &y), sub2[1](&x, &y), opt[0](&x), opt[1](&x));
}
