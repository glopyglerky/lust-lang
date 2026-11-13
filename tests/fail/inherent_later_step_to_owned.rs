// expect: L0643
// rustc picks core ToOwned for &S at the &&S step (returns &S), so the
// u32 annotation is E0308; lust cannot give that pick and rejects.
// hunt7: annotation form of probe_inherent_to_owned_refref.rs.
// rustc rejects: (&&v).to_owned() is &S, not u32.
#[derive(Debug, Clone)]
struct S(u32);
impl S {
    fn to_owned(&self) -> u32 {
        970 + self.0
    }
}
fn main() {
    let v = S(2);
    let r = &&v;
    let c: u32 = r.to_owned();
    println!("{}", c);
}
