// Two closures at bounds with their own lifetimes: a closure whose bound
// gives its output every region reaches the result only through those
// regions (the callee's signature), so `y` may end before `r` is used.
fn two<'a, 'b>(a: &'a u8, b: &'b u8, f: impl Fn(&'a u8) -> &'a u8, g: impl Fn(&'b u8) -> &'b u8) -> &'a u8 {
    g(b);
    f(a)
}

fn stat<'a, F: Fn() -> &'static u8, G: Fn(&'a u8) -> &'a u8>(a: &'a u8, f: F, g: G) -> &'static u8 {
    g(a);
    f()
}

static K: u8 = 7;

fn main() {
    let x = 1u8;
    let r;
    let s;
    {
        let y = 2u8;
        r = two(&x, &y, |p| p, |q| q);
        s = stat(&y, || &K, |q| q);
    }
    println!("{} {}", r, s);
}
