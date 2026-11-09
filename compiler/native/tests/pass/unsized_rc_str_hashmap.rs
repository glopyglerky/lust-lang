use std::collections::HashMap;
use std::rc::Rc;
fn main() {
    let mut m: HashMap<Rc<str>, u32> = HashMap::new();
    let k: Rc<str> = Rc::from("alpha");
    m.insert(k.clone(), 1);
    m.insert(Rc::from("beta"), 2);
    println!("{:?} {:?}", m.get("alpha"), m.get(&k));
    println!("{}", Rc::strong_count(&k));
    let mut keys: Vec<&Rc<str>> = m.keys().collect();
    keys.sort();
    println!("{:?}", keys);
}
