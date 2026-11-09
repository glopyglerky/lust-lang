// A closure body calls through its enclosing fn's `Fn` bound with the
// bound's lifetimes when they are `'static` or elided.
fn sum<P: Fn(&u8) -> &u8>(v: &[u8], p: &P) -> u8 {
    v.iter().map(|d| *p(d)).sum()
}

fn keep<'v, P: Fn(&u8) -> &u8>(v: &'v [u8], p: &P) -> Vec<&'v u8> {
    v.iter().map(|d| p(d)).collect()
}

fn stat<P: Fn(&'static str) -> usize>(p: &P) -> usize {
    let f = || p("lit");
    f()
}

fn ident(x: &u8) -> &u8 {
    x
}

fn len(s: &'static str) -> usize {
    s.len()
}

fn main() {
    let v = vec![1u8, 2];
    println!("{} {:?} {}", sum(&v, &ident), keep(&v, &ident), stat(&len));
}
