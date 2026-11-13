// `Self { .. }` and `Self::Variant { .. }` in const fns that a
// thread_local `const { .. }` initializer runs (ts-rust PerFileMap::new).
use std::cell::RefCell;
struct PerFileMap<V> {
    seen: usize,
    items: Vec<V>,
}
impl<V> PerFileMap<V> {
    const fn new() -> Self {
        Self { seen: 0, items: Vec::new() }
    }
}
enum Mode {
    Off,
    On { level: u8 },
}
impl Mode {
    const fn on(level: u8) -> Self {
        Self::On { level }
    }
}
thread_local! {
    static T: RefCell<PerFileMap<u8>> = const { RefCell::new(PerFileMap::new()) };
    static M: RefCell<Mode> = const { RefCell::new(Mode::on(3)) };
}
fn main() {
    T.with(|t| t.borrow_mut().items.push(4));
    T.with(|t| println!("{} {}", t.borrow().seen, t.borrow().items.len()));
    M.with(|m| match &*m.borrow() {
        Mode::On { level } => println!("on {}", level),
        Mode::Off => println!("off"),
    });
}
