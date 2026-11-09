// derive(Debug) and derive(Clone) over an `rc::Weak` field reached
// through a written std impl's bound (`OnceCell<T: Debug>`), as in
// ts-rust's ParsedCommandLine -> OnceCell<SourceAndOutputMaps> ->
// Rc<SourceOutputAndProjectReference> -> Weak<ParsedCommandLine>
// (TRIAGE R26). `rc::Weak<T>` is Debug and Clone for every T.
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

#[derive(Debug, Clone)]
struct Reference {
    source: String,
    resolved: Weak<CommandLine>,
}

#[derive(Debug, Default)]
struct Maps {
    by_source: HashMap<String, Rc<Reference>>,
}

#[derive(Debug)]
struct CommandLine {
    name: &'static str,
    maps: OnceCell<Maps>,
    names: OnceCell<Vec<String>>,
}

struct NoDebug;

#[derive(Debug, Clone)]
struct Holder {
    w: Weak<NoDebug>,
    cell: RefCell<Option<Weak<String>>>,
}

fn show<T: Debug>(t: &T) -> String {
    format!("{t:?}")
}

fn dup<T: Clone>(t: &T) -> T {
    t.clone()
}

fn main() {
    let cl = Rc::new(CommandLine { name: "tsconfig", maps: OnceCell::new(), names: OnceCell::new() });
    println!("{:?}", cl);
    let r = Rc::new(Reference { source: "a.ts".to_string(), resolved: Rc::downgrade(&cl) });
    let mut by_source = HashMap::new();
    by_source.insert("a.ts".to_string(), r.clone());
    cl.maps.set(Maps { by_source }).unwrap();
    println!("{:?}", cl);
    println!("{:#?}", cl.maps);
    let r2 = (*r).clone();
    println!("{} {}", r2.source, r2.resolved.upgrade().map(|c| c.name).unwrap_or("gone"));

    let s = Rc::new("kept".to_string());
    let h = Holder { w: Weak::new(), cell: RefCell::new(Some(Rc::downgrade(&s))) };
    let h2 = dup(&h);
    println!("{} {}", show(&h2), show(&OnceCell::from(h2.w.clone())));
    let d: Rc<dyn Fn() -> i32> = Rc::new(|| 3);
    let wd = Rc::downgrade(&d);
    println!("{} {:?}", show(&wd), dup(&wd).upgrade().map(|f| f()));
}
