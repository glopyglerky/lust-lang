// A derived `clone()` has the pointee's regions, not the borrow of the
// receiver: the clone outlives `s` (and `v2`) but not `x`.
#[derive(Clone, Debug)]
struct S<'a> { r: &'a String, n: u32 }
fn main() {
    let x = String::from("x");
    let c;
    {
        let s = S { r: &x, n: 1 };
        let t = &s;
        c = t.clone();
    }
    let v: Vec<&String> = vec![&x];
    let w;
    { let v2 = v.clone(); w = v2; }
    println!("{:?} {:?}", c, w);
}
