// capability: rejected: extern function types
// expect: L0602

type F = extern "C" fn(i32) -> i32;

fn main() {
    let f: Option<F> = None;
    println!("{}", f.is_none());
}
