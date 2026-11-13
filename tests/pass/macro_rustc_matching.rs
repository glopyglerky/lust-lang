// Calls that rustc's macro matcher accepts and the old matcher handled
// wrongly or not at all: one fragment parse per metavariable, run only
// where the metavariable alone can take the token.

// `ident` takes keywords and raw identifiers (never `_`).
macro_rules! name { ($i:ident) => { stringify!($i) }; }

// `meta` takes `=` and a whole expression.
macro_rules! meta { ($m:meta) => { stringify!($m) }; }

// An empty `vis` before an identifier, and the restricted forms.
macro_rules! vis { ($v:vis $i:ident) => { concat!("[", stringify!($v), "] ", stringify!($i)) }; }

// A separated repetition whose body can match nothing.
macro_rules! opt_list { ($($($x:ident)?),*) => { [$(concat!("<", $(stringify!($x),)? ">")),*] }; }

// A statement fragment: a lone `;` is the empty statement, and a braced
// item ends at its brace.
macro_rules! stmts { ($($s:stmt ;)*) => { [$(stringify!($s)),*] }; }

// `>>` splits for a type that ends inside it.
macro_rules! inner_ty { (< $t:ty >) => { stringify!($t) }; }

// Forwarded fragments: an `expr` goes on after a forwarded `expr`, a
// forwarded `path` can be called, and a `ty` that is a path stands in
// for a `path`.
macro_rules! expr { ($e:expr) => { $e }; }
macro_rules! add_one { ($e:expr) => { expr!($e + 1) }; }
macro_rules! call { ($p:path) => { expr!($p(20)) }; }
macro_rules! as_path { ($t:ty) => { path_name!($t) }; }
macro_rules! path_name { ($p:path) => { stringify!($p) }; }

// Struct patterns with shorthand fields, through `pat`.
struct P { x: u8, y: u8 }
macro_rules! pat_x { ($p:pat => $e:expr) => { match (P { x: 3, y: 4 }) { $p => $e, _ => 0 } }; }

fn double(v: u8) -> u8 { v * 2 }

fn main() {
    println!("{} {} {} {}", name!(fn), name!(self), name!(r#match), name!(union));
    println!("{}", meta!(a = 1 + 2));
    println!("{}", meta!(path::to(b, c)));
    println!("{}", vis!(x));
    println!("{}", vis!(pub(crate) y));
    println!("{}", vis!(pub(in crate) z));
    println!("{:?}", opt_list!(a, , b));
    println!("{:?}", stmts!(; ; let a = 1; fn h() {};));
    println!("{}", inner_ty!(<Vec<Vec<u8>>>));
    println!("{}", add_one!(2 * 3));
    println!("{}", call!(double));
    println!("{}", as_path!(std::vec::Vec<u8>));
    println!("{}", pat_x!(P { x, y: 4 } => x));
    println!("{}", pat_x!(P { ref x, .. } => *x + 1));
}
