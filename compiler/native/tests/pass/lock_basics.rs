use std::sync::{Mutex, RwLock};
#[derive(Debug, Default)]
struct S { a: u32, v: Vec<String> }
static G: Mutex<Option<u32>> = Mutex::new(None);
static H: Mutex<u64> = Mutex::new(7);
fn main() {
    let m = Mutex::new(5);
    {
        let mut g = m.lock().unwrap();
        *g += 1;
        println!("{}", *g);
        println!("{:?} {}", g, g);
    }
    println!("{:?}", m);
    println!("{:#?}", m);
    let r = RwLock::new(S::default());
    r.write().unwrap().v.push("x".to_string());
    {
        let a = r.read().unwrap();
        let b = r.read().unwrap();
        println!("{} {:?} {}", a.a, b.v, a.v.len());
        println!("{:?}", r);
    }
    *G.lock().unwrap() = Some(3);
    *H.lock().unwrap() += 4;
    println!("{}", H.lock().unwrap());
    println!("{:?}", G.lock().unwrap());
    let mut m2: Mutex<String> = Mutex::default();
    m2.get_mut().unwrap().push_str("hi");
    println!("{}", m2.into_inner().unwrap());
    println!("{}", m.is_poisoned());
    let n = m.into_inner().unwrap();
    println!("{n}");
    match r.read() { Ok(g) => println!("ok {}", g.v[0]), Err(e) => println!("err {e}") }
}
