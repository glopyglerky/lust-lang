// feature: supported.baseline
// form: elided refs, core derives, println/format/vec/assert macros
#[derive(Clone, Debug, PartialEq, Copy, Eq, Hash)]
struct P { x: i32 }
fn get(p: &P) -> &i32 { &p.x }
fn main() {
    let p = P { x: 3 };
    let q = p.clone();
    assert_eq!(p, q);
    let v = vec![1, 2, 3];
    println!("{} {:?} {}", get(&p), v, format!("n={}", v.len()));
}
