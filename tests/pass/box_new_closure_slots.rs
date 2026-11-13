// `Box::new(closure)` holds the closure itself: its captured borrow and
// the captured data's own regions stay apart (a `&mut` capture of a
// `Vec<&'static str>` need not be `'static`).
fn main() {
    let mut store: Vec<&'static str> = Vec::new();
    {
        let mut f: Box<dyn FnMut() + '_> = Box::new(|| store.push("lit"));
        f();
    }
    println!("{:?}", store);
}
