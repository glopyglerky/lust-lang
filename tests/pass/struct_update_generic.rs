// `..base` in a literal of a generic struct: the base has the literal's
// own type, so the written fields can be coercion sites (`Rc<closure>` to
// `Rc<dyn Fn>`), as in ts-rust's `NodeVisitorHooks { .., ..default() }`.
use std::rc::Rc;

pub type Hook<'a, C> = Rc<dyn Fn(u32, &mut C) -> u32 + 'a>;

pub struct Hooks<'a, C> {
    pub a: Option<Hook<'a, C>>,
    pub b: Option<Hook<'a, C>>,
}

impl<C> Default for Hooks<'_, C> {
    fn default() -> Self {
        Hooks { a: None, b: None }
    }
}

#[derive(Debug, Default, Clone)]
struct P<T> {
    x: T,
    y: T,
    tag: &'static str,
}

fn run<'a>(w: &'a mut u32) -> u32 {
    let hooks: Hooks<'a, u32> = Hooks {
        a: Some(Rc::new(|n: u32, c: &mut u32| -> u32 {
            *c += n;
            n
        })),
        ..Hooks::default()
    };
    (hooks.a.unwrap())(3, w) + hooks.b.map_or(0, |_| 1)
}

fn main() {
    let mut w = 1;
    println!("{} {}", run(&mut w), w);
    let base = P { x: 1u8, y: 2, tag: "base" };
    let p = P { x: 9, ..base.clone() };
    let q: P<String> = P { tag: "q", ..Default::default() };
    println!("{:?} {:?} {:?}", base, p, q);
}
