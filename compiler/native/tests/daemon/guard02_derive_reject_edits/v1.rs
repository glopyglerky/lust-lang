use std::collections::HashMap;

struct NoClone(u8);

#[derive(Clone)]
enum Any {
    Num(u8),
    Obj(HashMap<String, Any>),
    Raw(NoClone),
}

fn main() {
    let a = Any::Num(1);
    let b = a.clone();
    if let Any::Num(n) = b {
        println!("{n}");
    }
}
