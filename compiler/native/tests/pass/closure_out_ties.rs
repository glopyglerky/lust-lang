// A closure passed at an `Fn` bound gets the bound's return type: a
// named output lifetime takes what the closure returns, a `'static` one
// takes `'static` data, and an elided output is the only input lifetime.
fn app<'a, F: Fn(&'a u8) -> &u8>(x: &'a u8, f: F) -> &'a u8 {
    f(x)
}

fn get<'a, F: Fn() -> &'a u8>(f: F) -> &'a u8 {
    f()
}

fn hr<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    let x = 5u8;
    *f(&x)
}

fn run<F: Fn() -> &'static str>(f: F) -> usize {
    f().len()
}

struct H<'a>(&'a u8);

fn hidden<F: Fn(H) -> &u8>(f: F) -> u8 {
    let x = 9u8;
    *f(H(&x))
}

static K: u8 = 7;

fn main() {
    let k = 3u8;
    println!("{}", app(&k, |p| p));
    let r = get(|| &k);
    println!("{}", r);
    println!("{}", hr(|_| &K));
    let s: &'static str = "abc";
    println!("{}", run(move || s));
    println!("{}", hidden(|_| &K));
}
