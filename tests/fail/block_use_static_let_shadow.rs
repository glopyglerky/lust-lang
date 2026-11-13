// expect: L0110
// rustc E0530: let bindings cannot shadow statics (one a block `use` brings in).
mod m { pub static V2: i32 = 2000; }
fn main() {
    {
        let v = 166;
        println!("{}", v);
    }
    use m::V2 as v;
}
