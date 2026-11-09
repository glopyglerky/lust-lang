// Cow autoderef (W-STD-B): B's methods through the `&B` view, `*cow`,
// Cow<str> equality with the other string types, closures over Cows,
// and a user trait on Cow winning over the view.
use std::borrow::Cow;

fn go(s: &str) -> Cow<'_, str> {
    if s.starts_with("#") { Cow::Owned(s.replace("#", "+")) } else { Cow::Borrowed(s) }
}

fn bytes_of<'a>(s: &str, dflt: &'a [u8]) -> Cow<'a, [u8]> {
    let v: Vec<u8> = s.chars().map(|c| c as u8).collect();
    if v.len() > 2 { Cow::Owned(v) } else { Cow::Borrowed(dflt) }
}

fn main() {
    let a = go("#ab");
    let b = go("cd");
    println!("{} {}", a == "+ab", b != "cd");
    println!("{} {}", "+ab" == a, *"cd" == b);
    let s = String::from("cd");
    println!("{} {}", b == s, s == b);
    println!("{} {}", a == b, a < b);
    println!("{} {}", &a == "+ab", &&a == &"+ab");
    println!("{}", (*a).len() + (&*b).len());
    let r: &str = &*a;
    println!("{r}");
    let n = a.chars().filter(|c| *c == 'b').count();
    println!("{n}");
    let v = vec![go("#x"), go("y")];
    for c in &v {
        println!("{} {} {}", c.len(), c.is_empty(), c.to_uppercase());
    }
    let total: usize = v.iter().map(|c| c.len()).sum();
    println!("{total}");
    let seven = [7u8, 8];
    let k = bytes_of("hello", &seven);
    let m = bytes_of("h", &seven);
    println!("{} {} {:?}", k.len(), m.len(), k.to_vec());
    println!("{:?} {:?}", k.iter().max(), m.iter().copied().min());
    let f = |c: &Cow<str>| c.len() * 2;
    println!("{}", f(&a));
    let g = || b.len();
    println!("{}", g());
    println!("{}", a.trim_start_matches_len());
}

trait Ext {
    fn trim_start_matches_len(&self) -> usize;
}
impl Ext for Cow<'_, str> {
    fn trim_start_matches_len(&self) -> usize {
        self.len() + 100
    }
}
