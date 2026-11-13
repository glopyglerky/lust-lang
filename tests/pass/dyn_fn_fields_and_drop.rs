// `&dyn Fn` in struct fields and Vecs, and a closure env with a Drop
// type behind a `&dyn Fn` (the dyn reference does not own it).
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct V<'a> {
    name: &'a str,
    f: &'a dyn Fn(i32) -> i32,
}
fn walk(v: &V) -> i32 {
    (v.f)(3) + v.name.len() as i32
}
fn call0(f: &dyn Fn() -> usize) -> usize {
    f()
}
fn main() {
    let m = 10;
    let g = |n: i32| n * m;
    let h = |n: i32| n - 1;
    let v = V { name: "ab", f: &g };
    println!("{}", walk(&v));
    let fs: Vec<&dyn Fn(i32) -> i32> = vec![&g, &h];
    for f in &fs {
        println!("{}", f(5));
    }
    let total: i32 = fs.iter().map(|f| f(2)).sum();
    println!("{}", total);
    let d = D("env");
    let e = move || d.0.len();
    println!("{}", call0(&e));
    println!("after");
}
