// A closure passed at `Fn(&T) -> &U` gets rustc's deduced `for<'r>
// fn(&'r T) -> &'r U`: its body may return borrows of its argument
// (ts-rust `host_lookup(|c| &c.file_exists, ..)`).
struct Cache {
    a: Vec<u8>,
    b: Vec<u8>,
}

struct Host {
    cached: Cache,
}

impl Host {
    fn lookup(&self, map: impl Fn(&Cache) -> &Vec<u8>, k: usize) -> u8 {
        map(&self.cached)[k]
    }
}

fn app<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    let x = 5u8;
    *f(&x)
}

fn first<T, F: Fn(&[T]) -> &T>(v: &[T], f: F) -> &T {
    f(v)
}

fn main() {
    let h = Host { cached: Cache { a: vec![1, 2], b: vec![3] } };
    println!("{} {}", h.lookup(|c| &c.a, 1), h.lookup(|c| &c.b, 0));
    println!("{}", app(|x| x));
    let v = vec![String::from("p"), String::from("q")];
    println!("{}", first(&v, |s| &s[1]));
}
