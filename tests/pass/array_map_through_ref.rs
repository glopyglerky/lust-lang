// [T; N]::map through references copies a Copy array out (R22).
struct W { a: [u8; 3] }
fn main() {
    let w = W { a: [1, 2, 3] };
    let r = &w;
    let b = r.a.map(|x| x * 2);
    let rr = &&[5u16, 6];
    let c = rr.map(|x| x + 1);
    let mut m = [1i64, 2];
    let mr = &mut m;
    let d = mr.map(|x| -x);
    mr[0] = 9;
    println!("{:?} {:?} {:?} {:?}", b, c, d, m);
}
