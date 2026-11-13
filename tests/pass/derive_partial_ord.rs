// derive(PartialOrd) on structs and enums: all four operators, partial_cmp
// with float fields (NaN is unordered), enums ordered by discriminant
// value (rustc), and a derived PartialOrd next to a hand-written PartialEq.
#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
struct P { a: i32, b: f64 }
fn po_ops() {
    let v = [P { a: 1, b: 2.0 }, P { a: 1, b: f64::NAN }, P { a: 0, b: 9.0 }, P { a: 1, b: -0.0 }, P { a: 1, b: 0.0 }];
    for x in &v { for y in &v {
        print!("{}{}{}{} {:?} ", (x < y) as u8, (x <= y) as u8, (x > y) as u8, (x >= y) as u8, x.partial_cmp(y));
    } println!(); }
    println!("{} {} {} {}", v[0].lt(&v[1]), v[0].le(&v[0]), v[2].gt(&v[0]), v[3].ge(&v[4]));
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum E { A = 2, B = 1, C = 3, D = -5 }
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
#[repr(i8)]
enum G { A = 5, B = -3, C, D = 100 }
fn enum_discs() {
    let v = [E::A, E::B, E::C, E::D];
    for x in &v { for y in &v { print!("{:?} ", x.cmp(y)); } println!(); }
    let w = [G::A, G::B, G::C, G::D];
    for x in &w { for y in &w { print!("{} ", x < y); } println!(); }
}
#[derive(PartialOrd, Debug)]
struct S(f64);
impl PartialEq for S { fn eq(&self, _o: &S) -> bool { true } }
fn manual_eq() { println!("{:?} {} {}", S(1.0).partial_cmp(&S(2.0)), S(1.0) < S(2.0), S(f64::NAN) <= S(1.0)); }
fn main() {
    po_ops();
    enum_discs();
    manual_eq();
}
