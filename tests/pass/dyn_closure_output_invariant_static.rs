// A closure's inferred output `Option<Sender<Box<dyn Send>>>` holds an inferred object lifetime
// under the invariant `Sender`; the static's `'static` default reaches it through `get_or_init`
// or the fn's return type (ts-rust ast/file_version.rs free_in_background).
use std::sync::OnceLock;
use std::sync::mpsc::Sender;
fn queue() -> &'static Option<Sender<Box<dyn Send>>> {
    static QUEUE: OnceLock<Option<Sender<Box<dyn Send>>>> = OnceLock::new();
    QUEUE.get_or_init(|| {
        let (send, receive) = std::sync::mpsc::channel::<Box<dyn Send>>();
        std::thread::Builder::new()
            .name("free".to_string())
            .spawn(move || receive.into_iter().for_each(|b| std::mem::drop(b)))
            .ok()
            .map(|_| send)
    })
}
fn mk() -> Option<Sender<Box<dyn Send>>> {
    let f = || {
        let (send, _receive) = std::sync::mpsc::channel::<Box<dyn Send>>();
        Some(send)
    };
    f()
}
fn main() {
    let unsent = match queue() {
        Some(send) => send.send(Box::new(5u32)).err().map(|err| err.0),
        None => Some(Box::new(5u32) as Box<dyn Send>),
    };
    println!("{} {}", unsent.is_none(), mk().is_some());
}
