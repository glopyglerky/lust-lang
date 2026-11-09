// `Box::new`, `Rc::new`, and `String::from` as values: through
// iterator adapters, a generic Fn bound, and a plain call.
use std::rc::Rc;

fn apply<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}

fn main() {
    let names: Vec<String> = vec!["a", "bc", "d"].into_iter().map(String::from).collect();
    println!("{names:?}");
    let shared: Vec<Rc<String>> = names.into_iter().map(Rc::new).collect();
    println!("{} {}", shared.len(), Rc::strong_count(&shared[0]));
    let boxed: Vec<Box<u8>> = vec![1u8, 2].into_iter().map(Box::new).collect();
    println!("{boxed:?}");
    let from_chars: Vec<String> = vec!['x', 'y'].into_iter().map(String::from).collect();
    println!("{from_chars:?}");
    println!("{:?}", apply(Box::new, 3i32));
    println!("{}", apply(String::from, "hi"));
    let mk = Rc::<u64>::new;
    let r = mk(4);
    println!("{} {}", r, Rc::strong_count(&r));
    let s = String::from;
    let t: String = s('z');
    println!("{t}");
}
