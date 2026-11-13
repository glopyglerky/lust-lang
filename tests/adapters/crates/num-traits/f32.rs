// lust-reject: L0104
// The adapter leaves `to_f32` out, so the call rejects; the real crate
// gives Some(f32).
use num_traits::ToPrimitive;

fn main() {
    println!("{:?}", 0.1f64.to_f32());
}
