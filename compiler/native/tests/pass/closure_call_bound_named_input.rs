// Inside a closure, a call through an `Fn` bound whose argument names a
// lifetime of the enclosing fn (`'a`): the closure body has a universal for
// `'a` (End::Fn).
fn show(v: &u8) {
    println!("{}", v);
}

fn each<'a, P: Fn(&'a u8)>(x: &'a u8, p: &P) {
    let f = || p(x);
    f()
}

fn main() {
    let k = 1u8;
    each(&k, &show);
}
