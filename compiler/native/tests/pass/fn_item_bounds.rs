// A fn item passed for an `Fn` bound proves it from its signature:
// late-bound lifetimes are binders that take the lifetime the bound
// writes there or stay higher-ranked; `'static` meets `'static`.
static K: u8 = 4;

struct D {
    code: &'static str,
    msg: String,
}

fn path(d: &D) -> &'static str {
    d.code
}

fn by<P: Fn(&D) -> &'static str>(d: &D, p: &P) -> usize {
    p(d).len()
}

fn stat(_: &u8) -> &'static u8 {
    &K
}

fn ident(x: &u8) -> &u8 {
    x
}

fn idt<T>(x: &T) -> &T {
    x
}

fn run<F: Fn(&u8) -> &'static u8>(f: F) -> u8 {
    let x = 1u8;
    *f(&x)
}

fn hr<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    let x = 1u8;
    *f(&x)
}

fn first<'x, 'y>(a: &'x u8, _b: &'y u8) -> &'x u8 {
    a
}

fn pick<'b, G: Fn(&'b u8, &u8) -> &'b u8>(x: &'b u8, g: G) -> &'b u8 {
    let l = 2u8;
    g(x, &l)
}

fn main() {
    let d = D { code: "c1", msg: String::from("m") };
    println!("{} {}", by(&d, &path), d.msg);
    println!("{} {} {}", run(stat), hr(ident), hr(idt::<u8>));
    let k = 9u8;
    println!("{}", pick(&k, first));
}
