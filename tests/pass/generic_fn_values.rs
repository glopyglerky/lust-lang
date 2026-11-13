// Generic fns and generic methods as values: turbofish or inferred
// args, calls through the value, fn pointer coercion, Fn bounds,
// iterator adapters, and generic callers (template and instances).
fn id<T>(x: T) -> T {
    x
}
fn pair<A: Copy, B>(a: A, b: B) -> (A, A, B) {
    (a, a, b)
}
struct W<T> {
    v: T,
}
impl<T: Copy + std::fmt::Debug> W<T> {
    fn get(&self) -> T {
        self.v
    }
    fn wrap(v: T) -> W<T> {
        W { v }
    }
}
fn apply<F: Fn(u8) -> u8>(f: F, x: u8) -> u8 {
    f(x)
}
fn twice<T: Copy>(x: T) -> [T; 2] {
    [x, x]
}
fn generic_caller<T: Copy + std::fmt::Debug>(x: T) -> Vec<T> {
    let f = id::<T>;
    let g: fn(T) -> [T; 2] = twice;
    vec![f(x), g(x)[1]]
}
fn main() {
    let f = id::<u8>;
    println!("{}", f(5));
    let p = pair::<u8, &str>;
    println!("{:?}", p(1, "a"));
    let fp: fn(u16) -> u16 = id;
    println!("{}", fp(7));
    println!("{}", apply(id, 9));
    let ws = vec![W { v: 1u8 }, W { v: 2 }];
    let a: Vec<u8> = ws.iter().map(W::get).collect();
    println!("{:?}", a);
    let b: Vec<W<i64>> = vec![3i64, 4].into_iter().map(W::wrap).collect();
    println!("{}", b[1].get());
    let c: Vec<[u8; 2]> = vec![6u8].into_iter().map(twice).collect();
    println!("{:?}", c);
    println!("{:?}", generic_caller('z'));
    println!("{}", std::mem::size_of_val(&f));
    let h = W::<u32>::wrap;
    println!("{}", h(8).v);
}
