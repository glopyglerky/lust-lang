// Rc<dyn Trait> counts and ptr_eq with Rc's implicit weak reference,
// and drops that free the allocation (SANITIZE checks leaks).
use std::rc::Rc;

trait Show {
    fn show(&self) -> String;
}

struct A(u8);
struct B(String);

impl Show for A {
    fn show(&self) -> String {
        format!("A{}", self.0)
    }
}

impl Show for B {
    fn show(&self) -> String {
        self.0.clone()
    }
}

impl Drop for B {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let r: Rc<dyn Show> = Rc::new(A(1));
    let s = r.clone();
    let t: Rc<dyn Show> = Rc::new(A(1));
    println!("{} {} {}", Rc::strong_count(&r), Rc::weak_count(&s), Rc::ptr_eq(&r, &s));
    println!("{} {}", Rc::ptr_eq(&r, &t), r.show());
    drop(s);
    println!("{} {}", Rc::strong_count(&r), Rc::weak_count(&r));
    let v: Vec<Rc<dyn Show>> = vec![Rc::new(B("b1".to_string())), t, Rc::new(B("b2".to_string()))];
    let w = v[0].clone();
    drop(v);
    println!("{} {}", w.show(), Rc::strong_count(&w));
}
