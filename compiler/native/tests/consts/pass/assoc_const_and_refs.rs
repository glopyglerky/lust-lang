// ARCH 2.13: associated const lookup through the selected
// impl, references to other consts, string and byte literals,
// and promoted temporaries.
trait Kind {
    const TAG: u8;
    const NAME: &'static str;
}
struct Alpha;
struct Beta;
impl Kind for Alpha {
    const TAG: u8 = 1;
    const NAME: &'static str = "alpha";
}
impl Kind for Beta {
    const TAG: u8 = 2;
    const NAME: &'static str = "beta";
}
const TAGS: [u8; 2] = [Alpha::TAG, Beta::TAG];
const TEXT: &str = "hello";
const BYTES: &[u8] = b"abc";
const FIRST_TAG: &u8 = &TAGS[0];
// An assoc const used as an array length.
const PADDED: [u8; Beta::TAG as usize] = [7, 8];
fn main() {
    println!("{} {} {} {} {} {}", TAGS[1], TEXT, BYTES.len(), FIRST_TAG, PADDED[1], Alpha::NAME);
}
