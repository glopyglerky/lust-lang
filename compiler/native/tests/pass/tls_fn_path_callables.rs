// A std fn path as the callable: `KEY.with(Cell::get)` (as `get` is in
// std), `KEY.try_with(Cell::get)`, and `KEY.with(Rc::clone)`.
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static FLAG: Cell<bool> = const { Cell::new(true) };
    static N: std::cell::Cell<u32> = const { std::cell::Cell::new(9) };
    static SHARED: Rc<String> = Rc::new(String::from("shared"));
}

fn main() {
    if FLAG.with(Cell::get) {
        println!("flag");
    }
    println!("{}", N.with(std::cell::Cell::get));
    match N.try_with(Cell::get) {
        Ok(v) => println!("ok {}", v),
        Err(_) => println!("err"),
    }
    let a = SHARED.with(Rc::clone);
    let b = SHARED.with(Rc::clone);
    println!("{} {}", a, Rc::strong_count(&b));
}
