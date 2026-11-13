// rustc accepts: the old loan dies where its only holder is overwritten
// before any read, or where its holder is no longer live.
fn main() {
    let mut x = 1;
    let other = 5;
    let mut r = &x;
    r = &other;
    x = 2;
    println!("{} {}", r, x);

    let mut foo = 1;
    let bar = 2;
    let mut p: &i32;
    p = &foo;
    if x > 0 {
        println!("{}", *p);
        p = &bar;
        foo = 7;
    }
    println!("{} {}", *p, foo);

    let mut v = vec![1];
    let mut q = &v[0];
    println!("{}", q);
    q = &other;
    v.push(2);
    println!("{} {:?}", q, v);
}
