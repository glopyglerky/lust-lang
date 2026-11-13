fn seven() -> i32 {
    7
}
fn main() {
    let f: fn() -> i32 = seven;
    let o: Option<Box<fn() -> i32>> = None;
    println!("{}", o.is_some());
    let p: Option<Box<fn() -> i32>> = Some(Box::new(f));
    let g = *p.unwrap();
    println!("{}", g());
    let q: std::sync::OnceLock<Box<fn() -> i32>> = std::sync::OnceLock::new();
    q.get_or_init(|| Box::new(f));
    println!("{}", q.get().is_some());
    let b: Box<Box<fn() -> i32>> = Box::new(Box::new(f));
    let g3 = **b;
    println!("{}", g3());
    let o2: std::sync::OnceLock<std::sync::RwLock<Box<fn() -> i32>>> = std::sync::OnceLock::new();
    o2.get_or_init(|| std::sync::RwLock::new(Box::new(f)));
    println!("{}", o2.get().is_some());
}
