// A closure's creator adds only the region relations its body needs:
// `keep.push(*x)` needs the item's inner region to outlive `keep`'s
// elements, not the borrow of `v` (rustc accepts each).
fn push_all<'a>(src: &[&'a str], keep: &mut Vec<&'a str>) {
    let v: Vec<&'a str> = src.to_vec();
    v.iter().for_each(|x| keep.push(*x));
}
fn need(s: &'static str) -> usize {
    s.len()
}
fn main() {
    let s = String::from("a");
    let mut keep = Vec::new();
    push_all(&[s.as_str()], &mut keep);
    println!("{:?}", keep);
    let mut g: Vec<&'static str> = Vec::new();
    let mut f = |x: &'static str| g.push(x);
    f("a");
    let v: Vec<&'static str> = vec!["b"];
    v.iter().for_each(|x| f(x));
    drop(v);
    println!("{}", g.len());
    let w: Vec<&'static str> = vec!["c"];
    let mut n = 0;
    w.iter().for_each(|x| n += need(x));
    drop(w);
    println!("{}", n);
}
