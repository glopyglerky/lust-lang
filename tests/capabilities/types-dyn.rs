// capability: types: dyn Trait behind &, &mut, Box, Rc, and Arc, for crate traits and `Any`, `Error`, `Debug`, `Display`, and the Fn traits, with + Send / + Sync
// expect: ok

use std::any::Any;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::sync::Arc;

trait Shape {
    fn area(&self) -> u32;
}

struct Sq(u32);

impl Shape for Sq {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
}

fn fail() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("bad".into())
}

fn main() {
    let a: &dyn Shape = &Sq(1);
    let mut sq = Sq(2);
    let m: &mut dyn Shape = &mut sq;
    let b: Box<dyn Shape> = Box::new(Sq(3));
    let r: Rc<dyn Shape> = Rc::new(Sq(4));
    let s: Arc<dyn Shape + Send + Sync> = Arc::new(Sq(5));
    let any: Box<dyn Any> = Box::new(7u8);
    let d: &dyn Debug = &vec![1];
    let p: &dyn Display = &"text";
    let f: Box<dyn Fn(u32) -> u32> = Box::new(|x| x + 1);
    println!("{} {} {} {} {}", a.area(), m.area(), b.area(), r.area(), s.area());
    println!("{} {:?} {} {} {}", any.is::<u8>(), d, p, f(1), fail().unwrap_err());
}
