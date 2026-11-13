struct Z;
struct W(());
struct P {
    _p: std::marker::PhantomData<u8>,
}
fn main() {
    let z = Z;
    std::thread::scope(|s| {
        s.spawn(move || {
            let _r = &z;
        });
    });
    println!("ok");
    let w = W(());
    std::thread::scope(|s| {
        s.spawn(move || {
            let _r = &w;
        });
    });
    println!("ok2");
    let p = P { _p: std::marker::PhantomData };
    std::thread::scope(|s| {
        s.spawn(move || {
            let _r = &p;
        });
    });
    println!("ok3");
}
