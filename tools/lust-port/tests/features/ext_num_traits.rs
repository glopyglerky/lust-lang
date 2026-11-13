// feature: ext.num_traits
// form: ToPrimitive on f64 (to_i64, to_u32), as jsnum.rs
// cargo-dep: num-traits = "=0.2.19"
use num_traits::ToPrimitive;
fn main() {
    for x in [2.5f64, -1.0, 4294967296.0, f64::NAN, -0.9] {
        println!("{:?} {:?}", x.to_i64(), x.to_u32());
    }
}
