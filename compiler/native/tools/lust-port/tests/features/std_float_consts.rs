// feature: std.float_consts
// form: std::f64::consts and std::f32::consts by full path
fn main() {
    println!("{} {}", std::f64::consts::PI.to_bits(), std::f64::consts::E.to_bits());
    println!("{} {}", std::f32::consts::TAU.to_bits(), std::f64::consts::SQRT_2.to_bits());
    println!("{} {}", std::f64::consts::FRAC_1_PI.to_bits(), std::f32::consts::LN_2.to_bits());
}
