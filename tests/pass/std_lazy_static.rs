// `static X: LazyLock<T> = LazyLock::new(|| ..)`: the init closure moves
// into a hidden fn (expand/lazy.rs), the const evaluator stores it as a
// fn pointer (`ConstValue::FnPtr`), and first access runs it once. Fn
// items as init fns, `move ||`, `|| -> R`, nested modules, `Deref` method
// calls, `LazyLock::force` and `LazyLock::get` (W-STD-C).
use std::sync::LazyLock;
pub static WORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    println!("init words");
    let mut v = Vec::new();
    v.push("a");
    v.push("bc");
    v
});
static NAME: LazyLock<&str> = LazyLock::new(|| "nm");
static N: std::sync::LazyLock<u8> = std::sync::LazyLock::new(move || 5);
static M: LazyLock<u32> = LazyLock::new(|| -> u32 { 40 + 2 });
fn make() -> String { String::from("made") }
static S: LazyLock<String> = LazyLock::new(make);
static T: LazyLock<usize> = LazyLock::new(|| WORDS.len() * 10);
mod inner {
    pub static Q: std::sync::LazyLock<Vec<u16>> = std::sync::LazyLock::new(|| vec![1, 2]);
}
fn main() {
    println!("start");
    println!("{}", *T);
    println!("{:?} {}", *WORDS, WORDS.len());
    for w in WORDS.iter() { print!("{w} "); }
    println!();
    println!("{} {} {} {} {:?}", *NAME, *N + 1, *M, S.as_str(), *inner::Q);
    let r: &Vec<&str> = &WORDS;
    println!("{}", r[1]);
    println!("{:?}", LazyLock::force(&S));
}
