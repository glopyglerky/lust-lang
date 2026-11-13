// Derived PartialOrd/Ord/Hash on generic types: each type parameter gets
// the bound (rustc), the impls answer bounds in generic fns, and a generic
// type that derives both PartialOrd and Ord still compares its fields with
// partial_cmp (the Some(cmp) shortcut is for non-generic types only).
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(PartialEq, PartialOrd, Debug, Clone)]
struct W<T> { t: T, n: u8 }

fn need_hash<T: Hash + Eq>(_t: &T) -> u8 { 1 }
fn need_ord<T: Ord>(a: &T, b: &T) -> bool { a < b }
fn need_po<T: PartialOrd>(a: &T, b: &T) -> bool { a <= b }

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct K { a: u32, b: String }
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum E<T> { X, Y(T) }

#[derive(PartialEq, Eq, PartialOrd, Debug)]
struct Rev(u8);
impl Ord for Rev { fn cmp(&self, o: &Rev) -> Ordering { o.0.cmp(&self.0) } }
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct G<T>(T, u8);

fn main() {
    println!("{}", W { t: 3, n: 1 } < W { t: 3, n: 2 });
    println!("{:?}", W { t: String::from("b"), n: 0 }.partial_cmp(&W { t: String::from("a"), n: 9 }));
    println!("{:?}", W { t: (1u8, 2.5f64), n: 0 }.partial_cmp(&W { t: (1u8, f64::NAN), n: 0 }));
    println!("{}", W { t: Some(1u8), n: 0 } > W { t: None, n: 5 });
    println!("{}", W { t: W { t: 'a', n: 1 }, n: 0 } <= W { t: W { t: 'a', n: 1 }, n: 0 });
    let k1 = K { a: 1, b: String::new() };
    let k2 = K { a: 1, b: String::from("z") };
    println!("{} {} {}", need_hash(&k1), need_ord(&k1, &k2), need_po(&k2, &k1));
    println!("{} {} {}", need_hash(&E::Y(3u8)), need_ord(&E::Y(0i32), &E::Y(1i32)), need_po(&E::Y(2u8), &E::Y(1u8)));
    println!("{}", need_ord(&(1, String::from("a")), &(1, String::from("b"))));
    println!("{} {:?}", G(Rev(1), 0) < G(Rev(2), 0), G(Rev(1), 0).partial_cmp(&G(Rev(2), 0)));
}
