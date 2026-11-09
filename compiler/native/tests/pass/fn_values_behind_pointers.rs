// Fn items, fn pointers, and closures behind references and smart
// pointers: called through Fn-family bounds and directly.
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn seven() -> i32 { 7 }
fn once<F: FnOnce() -> i32>(f: F) -> i32 { f() }
fn twice<F: FnMut() -> i32>(mut f: F) -> i32 { f() + f() }
fn shared<F: Fn() -> i32>(f: F) -> i32 { f() + f() }
fn main() {
    let c = seven;
    let p: fn() -> i32 = seven;
    println!("{} {} {}", once(&&c), twice(Box::new(&&p)), shared(Box::new(Box::new(p))));
    println!("{} {}", once(&mut &c), shared(&Box::new(c)));
    let rp = &p;
    let w = Box::new(&rp);
    println!("{}", w());
    println!("{}", (Box::new(p))());
    let k = 5;
    let f = || k + 1;
    println!("{}", twice(&&f));
    let mut n = 0;
    let mut g = || {
        n += 1;
        n
    };
    println!("{}", twice(Box::new(&mut g)));
    println!("{}", twice(Box::new(g)));
    let mut d = D(1);
    let h = move || {
        d.0 += 1;
        d.0
    };
    println!("{}", twice(Box::new(h)));
    let e = D(9);
    let o = move || {
        let x = e;
        x.0
    };
    println!("{}", once(Box::new(o)));
    println!("end");
}
