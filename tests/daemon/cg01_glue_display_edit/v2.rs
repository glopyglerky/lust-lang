fn main() {
    println!("{}", P { a: 4 });
}
fn f0() {}
fn f1() {}
fn f2() {}
fn f3() {}
fn f4() {}
fn f5() {}
fn f6() {}
fn f7() {}
fn f8() {}
fn f9() {}
fn f10() {}
fn f11() {}
fn f12() {}
fn f13() {}
fn f14() {}
fn f15() {}
struct P { a: u8 }
impl std::fmt::Display for P {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "<{}>", self.a) }
}
