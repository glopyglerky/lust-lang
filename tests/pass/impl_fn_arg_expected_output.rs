// A generic fn or method that returns its closure's output, under an
// expected type: the closure body sees that type (rustc's expected
// inputs), so `sum`, `collect`, and `Default::default` resolve there.
struct T {
    v: Vec<u32>,
}
impl T {
    fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(self)
    }
    fn map_all<R>(&self, f: impl Fn(u32) -> R) -> Vec<R> {
        self.v.iter().map(|&x| f(x)).collect()
    }
}
fn with<R>(t: &T, f: impl FnOnce(&T) -> R) -> R {
    f(t)
}
fn with2<R, F: FnOnce(&T) -> R>(t: &T, f: F) -> R {
    f(t)
}
fn edit(o: &mut Vec<u32>, f: impl FnOnce(&mut Vec<u32>)) {
    f(o)
}
fn main() {
    let t = T { v: vec![1, 2, 3] };
    let s: u32 = with(&t, |x| x.v.iter().sum());
    let w: Vec<u64> = with2(&t, |x| x.v.iter().map(|&n| n as u64 * 2).collect());
    let n = with(&t, |x| x.v.len());
    let d: String = with(&t, |_| Default::default());
    println!("{} {:?} {} {:?}", s, w, n, d);
    let m = match n {
        0 => 5u32,
        _ => with(&t, |x| x.v.iter().sum()),
    };
    let s2: u32 = t.with(|x| x.v.iter().sum());
    let names: Vec<String> = t.map_all(|x| x.to_string());
    let mut o = vec![9];
    edit(&mut o, |v| v.push(4));
    println!("{} {} {:?} {:?}", m, s2, names, o);
}
