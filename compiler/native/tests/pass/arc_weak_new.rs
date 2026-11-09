use std::sync::Weak;
fn main() {
    let w: Weak<String> = Weak::new();
    let w2 = w.clone();
    println!("{} {} {} {:?}", w.strong_count(), w.weak_count(), w.ptr_eq(&w2), w.upgrade());
    let d: Weak<u8> = Weak::new();
    println!("{:?}", d);
}
