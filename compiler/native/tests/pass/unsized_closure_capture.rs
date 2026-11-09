use std::rc::Rc;
fn main() { let name: Rc<str> = Rc::from("cap"); let n2 = name.clone(); let f = move || format!("{}!", n2); println!("{} {} {}", f(), name, Rc::strong_count(&name)); drop(f); println!("{}", Rc::strong_count(&name)); }
