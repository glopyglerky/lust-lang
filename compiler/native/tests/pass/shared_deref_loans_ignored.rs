// A loan of a place behind a shared reference conflicts with nothing
// (rustc `ignore_borrow`): moving or reassigning the holder is fine (R12).
use std::collections::HashMap;
struct B { recs: &'static [u32] }
fn s1(b: B) -> (&'static [u32], B) { let r: &'static [u32] = &b.recs[..1]; (r, b) }
fn s3(b: B) -> (&'static u32, B) { let r: &'static u32 = &b.recs[0]; (r, b) }
enum Any { Obj(Vec<Any>), Num(u32) }
fn walk<'c>(cfg: &'c Vec<Any>) -> bool {
    let mut cur: Option<&'c Any> = None;
    for _ in 0..2 {
        let m = match cur { None => cfg, Some(Any::Obj(m)) => m, Some(_) => return false };
        cur = m.first();
    }
    cur.is_some()
}
fn get<'c>(h: &'c HashMap<String, u32>, cur: &mut Option<&'c u32>) { let m = h; *cur = m.get("a"); }
fn main() {
    static X: [u32; 2] = [1, 2];
    let mut h = HashMap::new();
    h.insert("a".to_string(), 3);
    let mut c = None;
    get(&h, &mut c);
    println!("{:?} {:?} {} {:?}", s1(B { recs: &X }).0, s3(B { recs: &X }).0, walk(&vec![Any::Obj(vec![Any::Num(1)])]), c);
}
