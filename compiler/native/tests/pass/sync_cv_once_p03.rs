use std::sync::OnceLock;
use std::thread;
static CONFIG: OnceLock<String> = OnceLock::new();
static N: OnceLock<u64> = OnceLock::new();
fn config() -> &'static str { CONFIG.get_or_init(|| { println!("init"); String::from("cfg") }) }
fn main() {
    println!("{:?} {:?}", CONFIG.get(), N);
    println!("{} {}", config(), config());
    let hs: Vec<_> = (0..4).map(|i| thread::spawn(move || *N.get_or_init(|| 40 + i * 0))).collect();
    for h in hs { print!("{} ", h.join().unwrap()); }
    println!();
    println!("{:?} {:?}", N.set(7), N.get());
    let mut o: OnceLock<Vec<u8>> = OnceLock::new();
    println!("{:?} {:?}", o, o.set(vec![1]));
    o.get_mut().unwrap().push(2);
    println!("{:?}", o.get());
    let a = o.take(); let b = o.take();
    println!("{:?} {:?}", a, b);
    let o2 = OnceLock::from(5);
    println!("{:?}", o2);
    println!("{:?}", o2.into_inner());
    let o3: OnceLock<String> = OnceLock::default();
    let s = String::from("keep");
    let v = o3.get_or_init(|| s.clone());
    println!("{} {}", v, o3.get_or_init(|| String::from("no")));
}
