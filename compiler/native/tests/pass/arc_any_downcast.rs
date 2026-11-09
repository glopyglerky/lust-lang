// `Arc<dyn Any + Send + Sync>::downcast` (R24, the ts-rust Context::value
// shape): a hit moves the Arc, a miss gives it back, counts and drops as in std.
use std::any::Any;
use std::sync::Arc;
#[derive(Debug)]
struct D(i32);
impl Drop for D { fn drop(&mut self) { println!("drop D{}", self.0); } }
fn main() {
    let a: Arc<dyn Any + Send + Sync> = Arc::new(D(1));
    let b = a.clone();
    println!("{}", Arc::strong_count(&a));
    match a.downcast::<String>() {
        Ok(s) => println!("string {s}"),
        Err(a) => {
            println!("not string {}", Arc::strong_count(&a));
            match a.downcast::<D>() {
                Ok(d) => println!("D {:?} {}", d, Arc::strong_count(&d)),
                Err(_) => println!("no"),
            }
        }
    }
    drop(b);
    let c: Arc<dyn Any + Send + Sync> = Arc::new(5u8);
    let r: Result<Arc<u8>, _> = c.downcast();
    println!("{:?}", r.map(|v| *v + 1).ok());
    println!("end");
}
