// More calls that follow rustc's macro matcher: raw identifiers and
// lifetimes are distinct tokens, `expr_2021` and `expr` fragments
// forward into each other, a forwarded `vis` heads an item, a path type
// takes a trailing `+`, and an expression ends before a macro call.

// A raw identifier or lifetime does not match the plain token.
macro_rules! raw { (foo) => { 1 }; ('a) => { 3 }; ($t:tt) => { 2 }; }

// Raw lifetimes, keywords included.
macro_rules! life { ($l:lifetime) => { stringify!($l) }; }
fn pick<'r#fn>(x: &'r#fn u8) -> &'r#fn u8 { x }

// `expr` and `expr_2021` fragments forward into each other.
macro_rules! e21 { ($e:expr_2021) => { e24!($e) }; }
macro_rules! e24 { ($e:expr) => { e21b!($e) }; }
macro_rules! e21b { ($e:expr_2021) => { $e * 2 }; }

// A forwarded `vis` is the visibility of the item after it.
macro_rules! item { ($i:item) => { $i }; }
macro_rules! with_vis { ($v:vis) => { item!($v fn seven() -> u8 { 7 }); }; }
with_vis!(pub(crate));

// A path type takes a `+` with no bound after it.
macro_rules! ty_plus { ($t:ty , $x:expr) => { $x }; ($($r:tt)*) => { 0 }; }

// An expression ends before a macro call that follows it.
macro_rules! two { ($a:expr_2021 => $b:tt) => { 1 }; ($a:ident $i:item) => { 2 }; }

// Where a fragment ends quietly, so the next arm gets its try: a struct
// pattern takes no range end and `::` before `{` ends a path. A doc
// comment's string is a literal.
macro_rules! soft { ($p:pat => x) => { 1 }; ($t:ty as x) => { 2 }; ($($r:tt)*) => { 3 }; }
macro_rules! doc { (#[doc = $s:literal] x) => { $s }; ($($t:tt)*) => { "tt" }; }

fn main() {
    println!("{} {} {} {} {}", raw!(r#foo), raw!(foo), raw!('r#a), raw!('a), raw!(r#a));
    println!("{} {}", life!('r#fn), life!('static));
    println!("{}", pick(&5));
    println!("{}", e21!(20 + 1));
    println!("{}", seven());
    println!("{} {}", ty_plus!(T +, 4), ty_plus!(Vec<u8> + Send, 5));
    println!("{}", two!(x m!();));
    println!("{} {}", soft!(S { a } ..= 1), soft!(a :: {b}));
    println!("{}", doc!(
        /// hello
        x
    ));
}
