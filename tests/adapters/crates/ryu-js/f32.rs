// lust-reject: L0612
// The adapter formats f64 only (`Float` has no f32 impl), so an f32
// rejects; the real crate prints the shortest f32 text.
fn main() {
    let mut b = ryu_js::Buffer::new();
    println!("{}", b.format(0.1f32));
}
