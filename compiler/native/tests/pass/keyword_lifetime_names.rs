// Keyword names in lifetime tokens: raw lifetimes and labels may use them,
// `'_` and `'static` are lifetimes, and token trees that no parse reads
// as a lifetime (stringify!, tt and lifetime fragments, macro call and
// attribute arguments) may hold any of them.
macro_rules! lt { ($l:lifetime) => { stringify!($l) } }
macro_rules! tts { ($($t:tt)*) => { stringify!($($t)*) } }
macro_rules! ty { ($t:ty) => { stringify!($t) } }
macro_rules! item { ($i:item) => { stringify!($i) } }
macro_rules! meta { ($m:meta) => { stringify!($m) } }
fn first<'r#fn>(x: &'r#fn u8, _y: &'_ u8) -> &'r#fn u8 { x }
fn name() -> &'static str { "static" }
fn main() {
    let v = 'r#loop: loop { break 'r#loop 7; };
    let w = 'r#gen: { break 'r#gen 8 };
    let u = 'union: loop { break 'union 9; };
    println!("{} {} {} {} {}", first(&1, &2), name(), v, w, u);
    println!("{}", stringify!('fn 'gen '_ 'static 'r#fn));
    println!("{} {}", lt!('fn), lt!('try));
    println!("{}", tts!('fn: loop {} break '_));
    println!("{}", ty!(n!(&'fn u8)));
    println!("{}", item!(#[doc('fn)] fn f() {}));
    println!("{}", meta!(a('fn, 'static)));
    println!("{}", ty!(&'r#fn u8));
}
