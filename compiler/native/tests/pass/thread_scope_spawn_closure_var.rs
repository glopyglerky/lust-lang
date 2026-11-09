fn main() {
    let x = 5;
    let f = move || x + 1;
    let r = std::thread::scope(|s| s.spawn(f).join().unwrap());
    println!("{}", r);
    let g = || 3;
    let r2 = std::thread::scope(|s| s.spawn(g).join().unwrap());
    println!("{}", r2);
    let t = (move || x + 2, 9);
    let r3 = std::thread::scope(|s| s.spawn(t.0).join().unwrap() + t.1);
    println!("{}", r3);
    println!("{}", f());
}
