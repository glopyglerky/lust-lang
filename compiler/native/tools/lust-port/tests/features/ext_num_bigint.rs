// feature: ext.num_bigint
// form: BigInt parse_bytes, pow, to_f64, and to_string, as jsnum.rs
// cargo-dep: num-bigint = "=0.4.6"
// cargo-dep: num-traits = "=0.2.19"
use num_bigint::BigInt;
use num_traits::ToPrimitive;
fn main() {
    let n = BigInt::parse_bytes(b"ff00ff00ff00ff00ff00", 16).unwrap();
    println!("{} {:?}", n, n.to_f64());
    let p = BigInt::from(-3i64).pow(41);
    println!("{} {:?}", p, p.to_f64());
}
