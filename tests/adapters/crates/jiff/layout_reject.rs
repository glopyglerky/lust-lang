// lust-reject: L0652
// The adapter's TimeZone does not have upstream's one-word layout, so
// lust must not answer size_of for it.
fn main() {
    println!("{}", std::mem::size_of::<jiff::tz::TimeZone>());
}
