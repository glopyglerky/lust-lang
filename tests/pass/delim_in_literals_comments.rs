// Delimiters inside strings, raw strings, byte strings, chars, comments and doc text do not count.
/// Doc with ( and ] and {.
#[doc = "attr doc ) ]"]
struct S;

// line comment ( [ {
/* block ) /* nested ] */ } */
const A: &str = "str ( [ {";
const R: &str = r#"raw ) " ] }"#;
const B: &[u8] = b"( ]";
const C: [char; 3] = ['(', ']', '}'];

fn main() {
    let _ = S;
    let n = match C[1] { ']' => 1, _ => 2 };
    println!("{} {} {} {:?} {}", A, R, B.len(), C, n);
}
