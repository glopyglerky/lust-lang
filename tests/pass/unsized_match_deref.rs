use std::rc::Rc;
fn main() { let r: Rc<str> = Rc::from("b"); match &*r { "a" => println!("A"), "b" => println!("B"), _ => println!("?") } let b: Box<str> = "zz".into(); if &*b == "zz" { println!("eq"); } }
