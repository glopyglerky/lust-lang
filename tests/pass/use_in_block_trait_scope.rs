// Block-scoped `use` items: a trait imported in a fn body (named,
// renamed, `as _`, or by glob) is in scope for method calls in the
// whole block: before the `use`, in nested blocks, in closures,
// and in nested fn items (W-BENCH2 bug 9). Other names (types,
// values, std items) bind in blocks too.
mod printer {
    pub trait EmitHost {
        fn name(&self) -> String;
    }
}
struct EmitHost;
impl printer::EmitHost for EmitHost {
    fn name(&self) -> String {
        "host".to_string()
    }
}
mod m {
    pub struct S(pub u32);
    pub fn f() -> u32 {
        7
    }
    pub const C: u32 = 5;
}
fn main() {
    // `as _` in the fn body (the W-BENCH2 repro).
    use printer::EmitHost as _;
    println!("{}", EmitHost.name());
    // A nested block, a closure, and a nested fn item all see it.
    let x = { EmitHost.name() };
    let g = || EmitHost.name();
    fn inner() -> String {
        EmitHost.name()
    }
    println!("{} {} {}", x, g(), inner());
    // Named and renamed imports, a glob, and plain names.
    {
        use m::{C, S, f};
        let s = S(3);
        println!("{} {} {}", s.0, f(), C);
    }
    {
        use printer::EmitHost as Host;
        let _ = Host::name(&EmitHost);
        println!("{}", EmitHost.name());
    }
    // The import covers the whole block, statements before it too.
    {
        println!("{}", EmitHost.name());
        use printer::EmitHost as _;
    }
    // std traits and std types by block `use`.
    {
        use std::fmt::Write as _;
        let mut s = String::new();
        let _ = s.write_str("ab");
        use std::io::Write as _;
        let mut v: Vec<u8> = Vec::new();
        let _ = v.write(&[65]);
        use std::collections::BTreeMap;
        let mut map = BTreeMap::new();
        map.insert(1u32, 2u32);
        println!("{} {} {:?}", s.len(), v.len(), map.get(&1));
    }
}
