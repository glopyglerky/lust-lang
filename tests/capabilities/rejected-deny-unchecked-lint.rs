// capability: rejected: `deny` and `forbid` of a lint lust does not check (named in the message)
// expect: L0531

#[deny(unreachable_code)]
fn main() {
    println!("x");
}
