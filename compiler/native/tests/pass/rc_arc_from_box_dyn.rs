// Rc/Arc from Box<dyn Tr>, Box<T>, Box<str>, Box<[T]> (alloc's
// `impl<T: ?Sized> From<Box<T>>`): the value moves into a new counted
// allocation, the vtable stays, and the value drops once, with the last
// strong pointer. Call, `.into()`, fn value, and generic forms.
use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

trait Speak {
    fn speak(&self) -> String;
}
struct Dog {
    name: String,
    _n: Noisy,
}
impl Speak for Dog {
    fn speak(&self) -> String {
        format!("woof {}", self.name)
    }
}
#[repr(align(64))]
struct Big(u8);
impl Speak for Big {
    fn speak(&self) -> String {
        format!("big {} {}", self.0, (self as *const Big as usize) % 64)
    }
}

type NpmInstall = Arc<dyn Fn(&str, &[String]) -> (Vec<u8>, Option<String>) + Send + Sync>;
struct Server {
    npm_install: Option<NpmInstall>,
}

fn via_bound<X: Into<Rc<dyn Speak>>>(x: X) -> Rc<dyn Speak> {
    x.into()
}

fn any_rc<T: ?Sized>(b: Box<T>) -> Rc<T> {
    Rc::from(b)
}

fn main() {
    // Box<dyn Tr> to Rc: counts, weak, clone, and the drop point.
    let b: Box<dyn Speak> = Box::new(Dog { name: "rex".to_string(), _n: Noisy("rex") });
    let r: Rc<dyn Speak> = Rc::from(b);
    let w = Rc::downgrade(&r);
    let r2 = r.clone();
    println!("{} {} {}", r.speak(), Rc::strong_count(&r), Rc::weak_count(&r));
    drop(r);
    println!("one left: {}", r2.speak());
    drop(r2);
    println!("upgrade none: {}", w.upgrade().is_none());

    // The ts-rust shape: an Option<Box<dyn Fn>> mapped into a field.
    let cap = Noisy("cap");
    let f: Box<dyn Fn(&str, &[String]) -> (Vec<u8>, Option<String>) + Send + Sync> =
        Box::new(move |c, a| {
            let _ = &cap;
            (c.as_bytes().to_vec(), a.first().cloned())
        });
    let s = Server { npm_install: Some(f).map(Arc::from) };
    let g = s.npm_install.clone().unwrap();
    println!("{:?} {}", g("cwd", &["i".to_string()]), Arc::strong_count(&g));
    drop(s);
    println!("server gone");
    drop(g);
    println!("closure gone");

    // A zero-sized closure, an over-aligned value, `.into()`, a generic bound.
    let z: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
    let z: Rc<dyn Fn(i32) -> i32> = z.into();
    println!("{}", z(21));
    let big: Box<dyn Speak> = Box::new(Big(7));
    println!("{}", via_bound(big).speak());

    // Downcast after the move.
    let a: Box<dyn Any> = Box::new(Dog { name: "any".to_string(), _n: Noisy("any") });
    let a: Rc<dyn Any> = a.into();
    match a.downcast::<Dog>() {
        Ok(d) => println!("dog {} {}", d.name, Rc::strong_count(&d)),
        Err(_) => println!("not a dog"),
    }
    let a: Box<dyn Any + Send + Sync> = Box::new(5u8);
    let a = Arc::<dyn Any + Send + Sync>::from(a);
    println!("{:?} {:?}", a.downcast_ref::<u8>(), a.downcast_ref::<i32>());

    // Sized, str, and slice boxes; a ?Sized generic.
    let n: Rc<i64> = Box::new(5).into();
    let d: Arc<Dog> = Arc::from(Box::new(Dog { name: "sized".to_string(), _n: Noisy("sized") }));
    let st: Arc<str> = Box::<str>::from("text").into();
    let sl: Rc<[Noisy]> = Rc::from(vec![Noisy("e1"), Noisy("e2")].into_boxed_slice());
    println!("{} {} {} {}", n, d.name, st, sl.len());
    drop(sl);
    let v: Vec<Rc<dyn Fn() -> usize>> = vec![Box::new(|| 1usize) as Box<dyn Fn() -> usize>, Box::new(|| 2)].into_iter().map(Rc::from).collect();
    println!("{}", v.iter().map(|f| f()).sum::<usize>());
    let local = String::from("borrowed");
    let lb: Box<dyn Fn() -> usize + '_> = Box::new(|| local.len());
    println!("{}", any_rc(lb)());
    println!("end");
}
