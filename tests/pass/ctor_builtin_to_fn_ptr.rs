// `Some` coerces to a fn pointer (codegen writes its fn, `Rvalue::ItemShim`).
fn main() {
    let f: fn(u32) -> Option<u32> = Some;
    println!("{:?}", f(1));
}
