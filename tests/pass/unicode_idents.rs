// Non-ASCII names that rustc accepts: CJK, Greek, a combining mark
// in the middle (U+0301 after x, which is already NFC), a
// precomposed letter (U+00E9), and raw identifiers.
fn main() {
    let 变量 = 3;
    let αβγ = 4;
    let x́ = 7;
    let café = 6;
    let r#α = 5;
    let r#match = 2;
    println!("{} {} {} {} {} {}", 变量, αβγ, x́, café, r#α, r#match);
}
