// Field access and inherent methods through Box's deref step.
struct P { x: i32, name: String }
impl P { fn get(&self) -> i32 { self.x * 2 } fn bump(&mut self) { self.x += 1; } }
fn main() {
    let mut p = Box::new(P { x: 20, name: "bo".to_string() });
    println!("{} {}", p.x, p.get());
    p.bump();
    p.x += 1;
    println!("{} {}", p.x, p.name);
    let q: Box<P> = p;
    println!("{}", q.get());
    let v = Box::new(vec![1, 2, 3]);
    println!("{} {:?}", v.len(), v[1]);
}
