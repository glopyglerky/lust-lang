// A `Box<String>` coerces to the error of `std::thread::Result<T>`
// (`Box<dyn Any + Send>`) in a program whose text has no `dyn` and no
// closure: the vtable discovery must still run (R26 sweep crash).
fn fail() -> std::thread::Result<u8> {
    Err(Box::new(String::from("lost")))
}

fn main() {
    let r: std::thread::Result<Vec<u8>> = Err(Box::new(String::from("e")));
    match r {
        Ok(_) => println!("ok"),
        Err(e) => println!("{:?} {:?}", e.downcast_ref::<u8>(), e.downcast_ref::<String>()),
    }
    match fail() {
        Ok(v) => println!("{v}"),
        Err(e) => println!("{:?}", e.downcast_ref::<String>()),
    }
}
