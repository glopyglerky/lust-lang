// rustc accepts: dropping a `Box<&i32>` does not read the reference, and a
// reborrow through a reference outlives reassigning that reference.
fn main() {
    let x = 5;
    let b: Box<&i32>;
    {
        let y = 6;
        b = Box::new(&y);
        println!("{}", b);
    }
    let p = 1;
    let q = 2;
    let mut r = &p;
    let s = &*r;
    r = &q;
    println!("{} {} {}", r, s, x);
}
