// Unicode char predicates, the trim family over Unicode whitespace,
// char::from_u32 / from_digit, and char::UNICODE_VERSION, on cases a
// reader can check by eye. The exhaustive proof is unicode_props_hash.

const LOW: bool = 'σ'.is_lowercase();
const UP: bool = 'Σ'.is_uppercase();
const WS: bool = '\u{a0}'.is_whitespace();
const CTL: bool = '\u{7f}'.is_control();
const DIG: bool = 'a'.is_digit(16);
const FD: Option<char> = char::from_digit(15, 16);
const VER: (u8, u8, u8) = char::UNICODE_VERSION;

const fn is_low(c: char) -> bool {
    c.is_lowercase()
}

const fn next_up(cp: u32) -> Option<char> {
    char::from_u32(cp)
}

fn show(c: char) {
    println!(
        "{:?} a={} n={} an={} lo={} up={} ws={} ctl={}",
        c,
        c.is_alphabetic(),
        c.is_numeric(),
        c.is_alphanumeric(),
        c.is_lowercase(),
        c.is_uppercase(),
        c.is_whitespace(),
        c.is_control()
    );
}

fn main() {
    println!("const {} {} {} {} {} {:?} {:?}", LOW, UP, WS, CTL, DIG, FD, VER);
    show('a');
    show('É');
    show('ß');
    show('µ');
    show('一');
    show('あ');
    show('5');
    show('²');
    show('½');
    show('\u{661}');
    show(' ');
    show('\u{a0}');
    show('\u{3000}');
    show('\u{200b}');
    show('\u{7}');
    show('\u{1f600}');
    show('\u{345}');
    // Associated-call form takes the char by value.
    println!("{} {}", char::is_alphabetic('é'), char::is_whitespace('\u{202f}'));
    println!("{:?} {:?} {:?}", char::from_u32(0x10FFFF), char::from_u32(0xD800), char::from_u32(0x110000));
    println!("{:?} {:?} {:?}", char::from_digit(0, 1), char::from_digit(9, 0), char::from_digit(35, 36));
    println!("{:?}", char::REPLACEMENT_CHARACTER);
    // Trim over every kind of Unicode whitespace.
    let s = "\u{a0}\u{3000}\u{2003} x \u{202f}\u{85}\u{1680}";
    println!("{:?} {:?} {:?}", s.trim(), s.trim_start(), s.trim_end());
    let t = String::from("\u{a0}hi\u{a0}");
    println!("{:?}", t.trim());
    // Predicates inside iterator closures (the closure gets &char).
    println!("{}", "aé 5!".chars().filter(|c| c.is_alphabetic()).count());
    println!("{}", "aé 5!".chars().filter(|c| c.is_alphanumeric()).count());
    let mut kept = String::new();
    for c in " x\u{a0}y ".chars() {
        if !c.is_whitespace() {
            kept.push(c);
        }
    }
    println!("{}", kept);
    // The const predicates also work inside const fn bodies.
    println!("{} {:?} {:?}", is_low('σ'), next_up(0xe9), next_up(0xD800));
}
