// Sol review probe `two_phase_activation_shared`: rustc accepts.
fn main() {
    let mut v = vec![1];
    v.push(v[0]);
    println!("{}", v.len());
}
