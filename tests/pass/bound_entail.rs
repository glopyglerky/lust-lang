// A type param passed for another generic fn's `Fn` bound proves it from
// its own bound: one lifetime both write is one region, a higher-ranked
// one may serve a named one, and a higher-ranked output follows the
// input the callee names.
fn inner<'b, G: Fn() -> &'b u8>(g: G) -> &'b u8 {
    g()
}

fn outer<'a, F: Fn() -> &'a u8>(f: F) -> &'a u8 {
    inner(f)
}

fn take<'b, G: Fn(&'b u8) -> &'b u8>(x: &'b u8, g: G) -> &'b u8 {
    g(x)
}

fn hr<'a, F: Fn(&u8) -> &u8>(x: &'a u8, f: F) -> &'a u8 {
    take(x, f)
}

fn fixed<G: FnMut(&'static str)>(mut g: G) {
    g("x");
}

fn general<F: FnMut(&str)>(f: F) {
    fixed(f)
}

fn by_ref<G: Fn(&u8)>(g: &G) {
    let x = 1u8;
    g(&x)
}

fn twice<F: Fn(&u8)>(f: F) {
    by_ref(&f);
    let c = || by_ref(&f);
    c();
}

fn by_mut<G: FnMut(&u8)>(g: &mut G) {
    let x = 1u8;
    g(&x)
}

fn count<F: FnMut(&u8)>(mut f: F) {
    by_mut(&mut f);
}

fn unit<F: Fn(&u8) -> ()>(f: F) {
    by_ref(&f)
}

fn main() {
    let k = 4u8;
    println!("{}", outer(|| &k));
    println!("{}", hr(&k, |p| p));
    general(|s| println!("{}", s));
    twice(|x| println!("{}", x));
    let mut n = 0u32;
    count(|x| n += *x as u32);
    println!("{}", n);
    unit(|x| println!("unit {}", x));
}
