// A trait default body is checked against the trait method's region
// signature (named lifetimes, `&self` elision), as rustc does.
trait Pick {
    fn pick<'a>(&self, x: &'a u8) -> &'a u8 {
        x
    }
    fn first<'a, 'b>(&self, x: &'a u8, _y: &'b u8) -> &'a u8 {
        x
    }
    fn me(&self) -> &Self {
        self
    }
    fn keep<'a>(&self, v: &mut Vec<&'a u8>, x: &'a u8) {
        v.push(x);
    }
}
struct S;
impl Pick for S {}
fn main() {
    let v = 3u8;
    let w = 4u8;
    let mut all: Vec<&u8> = Vec::new();
    S.keep(&mut all, &v);
    let _ = S.me();
    println!("{} {} {}", S.pick(&v), S.first(&v, &w), all.len());
}
