// A `let` extends the then block's tail temporary (edition 2024), and
// an expected type coerces inside the block, so these are valid.
static G: u8 = 9;
fn main() {
    let v = vec![1u8, 2];
    let k = v.len();
    let a = if k > 0 { &&G } else { &v[0] };
    let b: &u8 = if k > 5 { &&G } else { &v[1] };
    let f = |k: usize| -> &u8 { if k > 0 { &&G } else { &v[0] } };
    let mut c: &u8 = &0;
    c = if k > 1 { &&G } else { &v[0] };
    println!("{} {} {} {}", a, b, f(k), c);
}
