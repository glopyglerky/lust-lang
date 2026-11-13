struct S;
trait A {
    fn m(&self) -> u8;
}
impl A for S {
    fn m(&self) -> u8 {
        1
    }
}
impl A for &S {
    fn m(&self) -> u8 {
        2
    }
}
impl A for &&S {
    fn m(&self) -> u8 {
        3
    }
}
fn main() {
    let s = S;
    let r = &s;
    let rr = &r;
    println!("{} {} {}", s.m(), r.m(), rr.m());
}
