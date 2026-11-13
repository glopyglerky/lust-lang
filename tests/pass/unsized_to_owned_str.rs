use std::rc::Rc;
fn main() { let r: Rc<str> = Rc::from("own"); let s1 = String::from(&*r); let s2 = r.to_string(); let s3: String = (*r).to_owned(); let s4 = r.to_uppercase(); println!("{s1} {s2} {s3} {s4}"); }
