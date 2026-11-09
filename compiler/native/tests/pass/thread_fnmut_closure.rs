use std::thread;
fn main() {
    let mut v = vec![1, 2, 3];
    let h = thread::spawn(move || { v.push(4); v.iter().sum::<i32>() });
    println!("{}", h.join().unwrap());
    let w = vec![String::from("a"), String::from("b")];
    let h = thread::spawn(move || w.len());
    println!("{}", h.join().unwrap());
    let r = thread::spawn(|| thread::spawn(|| 5).join().unwrap() * 2).join();
    println!("{:?}", r);
}
