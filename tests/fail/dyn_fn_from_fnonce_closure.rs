// expect: L0612
// rustc E0507: a closure that moves a capture out is not `dyn Fn`.
fn call0(f: &dyn Fn()) { f() }
fn main() { let s = String::new(); call0(&move || drop(s)); }
