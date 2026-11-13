// str methods: strip_prefix and strip_suffix, trim_start and
// trim_end, replace (its one-byte fast path and general path
// leave different capacities, as in Rust), repeat, and
// is_char_boundary.
fn main() {
    println!("{:?}", "hello".strip_prefix("he"));
    println!("{:?}", "hello".strip_suffix("lo"));
    println!("{:?}", "hello".strip_prefix("x"));
    println!("{:?}", "hello".strip_prefix("hello"));
    let owned = "héllo".to_string();
    println!("{:?}", owned.strip_prefix("hé"));
    println!("{:?} {:?}", "  x y  ".trim_start(), "  x y  ".trim_end());
    println!("{:?}", "\t\n abc\u{3000}".trim_start());
    println!("{}", "hello".replace("x", "y").capacity());
    println!("{}", "hello world".replace("o", "0").capacity());
    println!("{}", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".replace("aa", "b").capacity());
    println!("{}", "xxxxxxxxxxxxxxxxxxxx".replace("x", "y").capacity());
    println!("{}", "aaaaaaaaaaaaaaaaaaaa".replace("a", "bb").capacity());
    println!("{}", "éééééééééééééééééééé".replace("é", "e").capacity());
    println!("{:?} {}", "ab".replace("", "-"), "ab".replace("", "-").capacity());
    println!("{:?} {}", "aaa".replace("a", ""), "aaa".replace("a", "").capacity());
    println!("{:?}", "αβ".replace("β", "c"));
    println!("{:?}", "one two one".replace("one", "three"));
    println!("{:?} {}", "ab".repeat(3), "ab".repeat(3).capacity());
    println!("{}", "hello".repeat(1).capacity());
    println!("{}", "".repeat(5).capacity());
    let s = "héllo";
    for i in 0..8 {
        println!("{} {}", i, s.is_char_boundary(i));
    }
}
