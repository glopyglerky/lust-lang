// Comparison trait methods named through their trait, in generic code
// at builtin and user types and in concrete code.
fn f<T: PartialEq>(t: &T) -> bool { <T as PartialEq>::eq(t, t) }
fn g<T: PartialOrd>(a: &T, b: &T) -> bool { <T as PartialOrd>::lt(a, b) }
fn h<T: Clone>(a: &T) -> T { <T as Clone>::clone(a) }
fn k<T: PartialEq>(t: &T) -> bool { PartialEq::ne(t, t) }
fn m<T: Ord>(a: &T, b: &T) -> std::cmp::Ordering { Ord::cmp(a, b) }
fn main() {
    println!("{} {}", f(&3u8), f(&String::from("a")));
    println!("{}", f(&vec![1, 2]));
    println!("{} {}", g(&1.5f64, &2.0), g(&"a", &"b"));
    println!("{} {}", h(&7i32), h(&String::from("x")));
    println!("{}", k(&'c'));
    println!("{:?}", m(&3u64, &2));
}
