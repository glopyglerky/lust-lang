// Option::as_deref over Rc and Arc payloads, sized and unsized
// (TRIAGE R26: ts-rust calls it on Option<Rc<T>>, Option<Arc<[T]>>,
// Option<Rc<dyn Fn>>, and Option<Rc<Arc<T>>>), and over references
// to unsized types.
use std::rc::Rc;
use std::sync::Arc;

struct Options {
    strict: bool,
}

struct Program {
    options: Option<Arc<Options>>,
    filter: Option<Rc<dyn Fn(&str) -> bool>>,
    calls: Option<Arc<[u32]>>,
}

fn strict(p: &Program) -> bool {
    p.options.as_deref().is_some_and(|o| o.strict)
}

fn calls<'a>(p: &'a Program) -> &'a [u32] {
    p.calls.as_deref().unwrap_or(&[])
}

fn main() {
    let p = Program {
        options: Some(Arc::new(Options { strict: true })),
        filter: Some(Rc::new(|s: &str| s.ends_with(".ts"))),
        calls: Some(Arc::from(vec![3, 1, 2])),
    };
    println!("{} {:?}", strict(&p), calls(&p));
    println!("{:?}", p.filter.as_deref().map(|f| (f("a.ts"), f("b.js"))));
    let q = Program { options: None, filter: None, calls: None };
    println!("{} {:?} {}", strict(&q), calls(&q), q.filter.as_deref().is_none());

    let name: Option<Rc<str>> = Some(Rc::from("root"));
    let nested: Option<Rc<Arc<String>>> = Some(Rc::new(Arc::new("inner".to_string())));
    println!("{} {:?}", name.as_deref().unwrap_or("-"), nested.as_deref().map(|a| a.len()));

    let mut count = 0;
    let mut bump = || count += 1;
    let mut f: Option<&mut dyn FnMut()> = Some(&mut bump);
    if let Some(g) = f.as_deref_mut() {
        g();
        g();
    }
    let r: Option<&dyn Fn() -> i32> = Some(&|| 9);
    println!("{:?}", r.as_deref().map(|g| g()));
    drop(f);
    println!("{count}");
}
