// expect: L0110
// rustc E0530: for bindings cannot shadow statics (one a block `use` brings in).
mod m { pub static V2: i32 = 2000; }
fn main() {
    println!("{}", v);
    use m::V2 as v;
    {
        for v in 498..503 { println!("f{}", v); break; }
    }
}
