// Removed types and statics leave holes in the session's Adt and static
// tables: the bodies that stay keep their cached C, which names the other
// types and statics by index. Each version removes or restores some.
struct Beta {
    b: i64,
}
struct Gamma {
    c: u8,
    d: u16,
}
static S2: i64 = 20;
static S3: u8 = 30;

fn use_gamma() -> u32 {
    let g = Gamma { c: S3, d: 7 };
    g.c as u32 + g.d as u32
}

fn tag() -> &'static str {
    static T: &str = "t";
    T
}

fn use_beta() -> i64 {
    Beta { b: S2 }.b
}

fn main() {
    println!("{} {} {}", use_gamma(), tag(), use_beta());
}
