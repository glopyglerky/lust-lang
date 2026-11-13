// lust-reject: L0652
// BigInt's layout is not upstream's (no exact-layout entry), so lust
// must not answer size_of for it.
fn main() {
    println!("{}", std::mem::size_of::<num_bigint::BigInt>());
}
