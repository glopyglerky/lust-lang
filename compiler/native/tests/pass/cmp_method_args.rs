// Comparison method calls whose argument rustc deref-coerces: `Self`
// with one impl of the trait takes `&Self` through references.
fn main() {
    let a = 5i32;
    let r = &a;
    println!("{:?} {:?}", a.eq(&&a), r.partial_cmp(&&&a));
    let x = "a";
    let rx = &x;
    println!("{:?} {:?}", x.eq(x), rx.partial_cmp(&rx));
    let s = String::from("a");
    println!("{} {:?}", s.eq(&s), s.partial_cmp(&&s));
    let v = vec![1u8];
    println!("{:?}", v.partial_cmp(&&v));
    let b = Box::new(3u8);
    println!("{} {:?}", b.eq(&&b), b.cmp(&b));
    let o = Some(1u8);
    println!("{}", o.eq(&&o));
    println!("{:?} {}", 5.partial_cmp(&6), 1.eq(&2u8));
    let d = std::time::Duration::ZERO;
    println!("{}", d.eq(&&d));
}
