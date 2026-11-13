// `format!` capacity follows fmt::Arguments::estimated_capacity after
// rustc inlines literal `{}` arguments, and `to_string` follows std's
// specializations (signed integers preallocate, unsigned and the
// text types copy exactly, everything else grows from empty).
fn main() {
    let x = 5;
    let s = "hello";
    let v = vec![1, 2, 3];
    println!("{}", format!("{}", x).capacity());
    println!("{}", format!("ab{}", x).capacity());
    println!("{}", format!("{}ab", x).capacity());
    println!("{}", format!("{}abcdefghijklmnopq", x).capacity());
    println!("{}", format!("abc").capacity());
    println!("{}", format!("").capacity());
    println!("{}", format!("a{}b{}c", x, x).capacity());
    println!("{}", format!("{}{}", s, s).capacity());
    println!("{}", format!("{:?}", v).capacity());
    println!("{}", format!("{x:>10}").capacity());
    println!("{}", format!("{:>10}", 1.5).capacity());
    println!("{}", format!("{}", 1234567.89).capacity());
    println!("{}", format!("{{}}").capacity());
    println!("{}", format!("{{{}}}", x).capacity());
    println!("{}", format!("abcdefgh{}", "0123456789abcdefghij").capacity());
    println!("{}", format!("abcdefghijklmnop{}", 1_000).capacity());
    println!("{}", format!("abcdefghijklmnop{}", &&"xy").capacity());
    println!("{}", format!("abcdefghijklmnop{}", 0x10).capacity());
    println!("{}", format!("abcdefghijklmnop{}", -5).capacity());
    println!("{}", format!("abcdefghijklmnop{}", 'c').capacity());
    println!("{}", format!("abcdefghijklmnop{}", true).capacity());
    println!("{}", format!("abcdefghijklmnop{:?}", "xy").capacity());
    println!("{}", format!("{} {}", 1, 2).capacity());
    println!("{}", format!("{:?}", "a long string with no escapes inside").capacity());
    println!("{}", format!("{:?}", "tab\there\nnew\"q").capacity());
    println!("{} {} {}", 5i32.to_string().capacity(), (-5i32).to_string().capacity(), 1234567i64.to_string().capacity());
    println!("{} {} {} {}", 5u8.to_string().capacity(), 123u8.to_string().capacity(), 5i8.to_string().capacity(), (-128i8).to_string().capacity());
    println!("{} {} {}", 5i128.to_string().capacity(), (-5i128).to_string().capacity(), 5u128.to_string().capacity());
    println!("{} {} {}", 'é'.to_string().capacity(), true.to_string().capacity(), false.to_string().capacity());
    println!("{} {}", 1.5f64.to_string().capacity(), 12345678.9f64.to_string().capacity());
    let st = String::from("abcdef");
    println!("{} {}", st.to_string().capacity(), (&st).to_string().capacity());
    let n = 7i64;
    let rn = &n;
    let rrn = &rn;
    println!("{} {}", rn.to_string().capacity(), rrn.to_string().capacity());
}
