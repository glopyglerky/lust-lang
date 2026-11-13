// A closure body that calls through the enclosing fn's iterator (`Item = &'a u32`) returns `'a`
// items: the fn's `'a` may reach the closure's output (tf-borrow gen54).
fn f<'a, I: Iterator<Item = &'a u32>>(mut it: I, out: &mut Vec<&'a u32>) {
    let mut g = || it.next(); while let Some(x) = g() { out.push(x); }
}
fn main() {
    let n = [1u32, 2, 3];
    let mut v = Vec::new();
    {
        f(n.iter(), &mut v);
    }
    println!("{}", v.len());
}
