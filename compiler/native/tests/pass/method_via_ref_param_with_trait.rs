trait Marker {}
struct S {
    n: u32,
}
impl S {
    fn get(&self) -> u32 {
        self.n
    }
}
fn via_ref(r: &S) -> u32 {
    r.get()
}
fn via_ref_ref(rr: &&S) -> u32 {
    rr.get()
}
fn main() {
    let s = S { n: 42 };
    let r = &s;
    println!("{} {} {}", via_ref(r), via_ref_ref(&r), r.get());
}
