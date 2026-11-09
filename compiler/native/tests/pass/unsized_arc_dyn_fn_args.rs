use std::sync::Arc;
type Cb = Arc<dyn Fn(&str) -> usize + Send + Sync>;
fn apply(f: &Cb, s: &str) -> usize { f(s) }
fn main() {
    let k = 3;
    let f: Cb = Arc::new(move |s: &str| s.len() + k);
    let g = f.clone();
    println!("{} {}", apply(&f, "abcd"), g("x"));
    let h: Arc<dyn Fn(u32) -> u32> = Arc::new(|x| x * 2);
    println!("{}", h(21));
}
