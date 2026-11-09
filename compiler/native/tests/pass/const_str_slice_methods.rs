const GO_STRING_MARKER: char = '\u{FDD0}';
const GO_STRING_MARKER_BYTES: &[u8] = "\u{FDD0}".as_bytes();
const MERGE_CONFLICT_MARKER_LENGTH: usize = "<<<<<<<".len();
const MARKER_LEN: usize = GO_STRING_MARKER_BYTES.len();
const FIRST_MARKER: u8 = GO_STRING_MARKER_BYTES[0];
const TEXT: &str = "héllo";
const TEXT_BYTES: &[u8] = TEXT.as_bytes();
const TEXT_LEN: usize = TEXT.len();
const EMPTY: bool = "".is_empty();
const NOT_EMPTY: bool = "x".is_empty();
const ASCII: bool = "abc".is_ascii();
const NOT_ASCII: bool = "hé".is_ascii();
const ARR: [u8; 3] = [10, 20, 30];
const ARR_LEN: usize = ARR.len();
const ARR_MID: u8 = ARR[1];
const BYTES: &[u8; 16] = b"0123456789abcdef";
const BYTES_LEN: usize = BYTES.len();
const SUGAR: [u8; 6] = [7; TEXT_LEN];

static ASCII_IDENT_PART: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0;
    while i < 128 {
        let b = i as u8;
        table[i] = b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
        i += 1;
    }
    table
};

const fn byte_len(s: &[u8]) -> usize {
    s.len()
}

const fn key_ok(key: &str) -> bool {
    !key.is_empty()
}

const FROM_FN: usize = byte_len("abcd".as_bytes());
const KEY_OK: bool = key_ok("k");
const DIGIT: Option<u32> = '9'.to_digit(10);
const CH_A: Option<char> = char::from_u32(97);
const CH_BAD: Option<char> = char::from_u32(0xD800);
const CH_MAX: Option<char> = char::from_u32(0x10FFFF);
const LOWER_Z: u8 = b'Z'.to_ascii_lowercase();
const A_ALPHA: bool = b'a'.is_ascii_alphabetic();

fn main() {
    println!("{:?} {} {} {}", GO_STRING_MARKER_BYTES, MARKER_LEN, FIRST_MARKER, MERGE_CONFLICT_MARKER_LENGTH);
    println!("{} {:?} {} {}", GO_STRING_MARKER as u32, TEXT_BYTES, TEXT_LEN, SUGAR.len());
    println!("{} {} {} {}", EMPTY, NOT_EMPTY, ASCII, NOT_ASCII);
    println!("{} {} {}", ARR_LEN, ARR_MID, BYTES_LEN);
    println!("{} {} {} {}", ASCII_IDENT_PART[b'a' as usize], ASCII_IDENT_PART[b' ' as usize], FROM_FN, KEY_OK);
    println!("{:?} {:?} {:?} {:?}", DIGIT, CH_A, CH_BAD, CH_MAX);
    println!("{} {} {}", 'é'.len_utf8(), '\u{1F600}'.len_utf16(), 'A'.to_ascii_lowercase());
    println!("{} {}", LOWER_Z, A_ALPHA);
}
