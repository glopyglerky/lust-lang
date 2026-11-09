// Owned pointers to unsized types are fat: two words.
fn main() {
    println!(
        "{} {} {} {} {} {}",
        std::mem::size_of::<Box<str>>(),
        std::mem::size_of::<Box<[u8]>>(),
        std::mem::size_of::<std::rc::Rc<str>>(),
        std::mem::size_of::<std::rc::Rc<[u8]>>(),
        std::mem::size_of::<std::sync::Arc<str>>(),
        std::mem::size_of::<std::sync::Arc<[u8]>>()
    );
    let b: Box<str> = Box::from("hi");
    let r: std::rc::Rc<str> = std::rc::Rc::from("yo");
    println!("{b} {r}");
}
