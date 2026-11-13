// A concrete UFCS comparison at a builtin type, and a derived one.
#[derive(PartialEq, PartialOrd, Clone, Debug)]
struct P(u8);
fn f<T: PartialEq>(t: &T) -> bool { <T as PartialEq>::eq(t, t) }
fn main() {
    println!("{}", <u8 as PartialEq>::eq(&1, &2));
    println!("{}", PartialEq::eq(&1u8, &1));
    println!("{}", <String as PartialEq>::eq(&String::new(), &String::new()));
    println!("{}", <u8 as PartialOrd>::lt(&1, &2));
    println!("{}", <u8 as Clone>::clone(&5));
    println!("{}", f(&P(1)));
}
