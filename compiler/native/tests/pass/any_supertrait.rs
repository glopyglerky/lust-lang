// A user trait with Any as a supertrait: as_any, trait upcasting to dyn Any, Rc<dyn Any>::downcast, panic payload shapes (W-CHECK-G3).
use std::any::Any;
use std::rc::Rc;
trait AnyValue: Any { fn as_any(&self) -> &dyn Any; fn name(&self) -> String; }
impl<T: Any + Tag> AnyValue for T {
    fn as_any(&self) -> &dyn Any { self }
    fn name(&self) -> String { self.tag() }
}
trait Tag { fn tag(&self) -> String; }
struct P { x: i32 }
impl Tag for P { fn tag(&self) -> String { format!("P{}", self.x) } }
impl Tag for u64 { fn tag(&self) -> String { format!("u{}", self) } }
fn payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(m) = payload.downcast_ref::<&str>() { m.to_string() }
    else if let Some(m) = payload.downcast_ref::<String>() { m.clone() }
    else { "unknown".to_string() }
}
fn id_of<T: Any>(x: &T) -> bool { x.type_id() == std::any::TypeId::of::<u64>() }
fn main() {
    let v: Vec<Box<dyn AnyValue>> = vec![Box::new(P { x: 4 }), Box::new(9u64)];
    for item in &v {
        println!("{} {:?}", (**item).name(), (**item).as_any().downcast_ref::<P>().is_some());
        let a: &dyn Any = (**item).as_any();
        println!("{}", a.is::<u64>());
        let up: &dyn Any = &**item;
        println!("{}", up.is::<u64>());
    }
    let rc: Rc<dyn Any> = Rc::new(P { x: 2 });
    let rc2 = rc.clone();
    match rc.downcast::<P>() { Ok(p) => println!("rc {} {}", p.x, Rc::strong_count(&p)), Err(_) => println!("no") }
    println!("{:?}", rc2.downcast_ref::<P>().unwrap().x);
    let b: Box<dyn Any + Send> = Box::new(String::from("boom"));
    println!("{}", payload_message(&*b));
    println!("{} {}", id_of(&5u64), id_of(&5u32));
    println!("{:?}", &*b as &dyn Any);
    println!("{:?}", b);
}
