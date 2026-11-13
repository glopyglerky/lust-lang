// Primitive type paths as calls and as fn values (TRIAGE T12):
// `u8::is_ascii_digit`, `str::len`, `i32::abs`, `char::len_utf8`,
// `bool::then_some`, `String::as_str`, and `u64::from`, direct and
// through iterator adapters.
fn read<T>(v: T) -> T {
    v
}

fn main() {
    println!("{} {}", u8::is_ascii_digit(&b'5'), u8::is_ascii_alphabetic(&b'x'));
    println!("{} {} {}", i32::abs(read(-3)), u32::is_power_of_two(read(8u32)), i64::signum(read(-9i64)));
    println!("{} {}", str::len("abcd"), str::is_empty(""));
    println!("{} {} {}", char::is_alphabetic(read('x')), char::is_ascii_digit(&read('7')), char::len_utf8(read('é')));
    println!("{:?}", bool::then_some(read(true), 4));
    println!("{}", "12ab".bytes().take_while(u8::is_ascii_digit).count());
    println!("{}", b"a1b2".iter().all(u8::is_ascii_alphanumeric));
    println!("{:?}", [1u16, 2].iter().copied().map(u64::from).collect::<Vec<u64>>());
    println!("{}", [3u32, 4].iter().copied().map(u64::from).sum::<u64>());
    println!("{:?}", "a/b".split('/').map(str::to_string).collect::<Vec<String>>());
    println!("{:?}", ["a", "bb"].iter().copied().map(str::len).collect::<Vec<usize>>());
    println!("{:?}", [-1i32, 2].iter().copied().map(i32::abs).collect::<Vec<i32>>());
    println!("{:?}", "éx".chars().map(char::len_utf8).collect::<Vec<usize>>());
    println!("{}", "a1b2".chars().filter(char::is_ascii_digit).count());
    println!("{:?}", "a1".chars().map(char::is_alphabetic).collect::<Vec<bool>>());
    let v = vec![String::from("hi"), String::from("yo")];
    println!("{:?}", v.iter().map(String::as_str).collect::<Vec<&str>>());
    println!("{:?}", v.iter().map(String::len).collect::<Vec<usize>>());
    let f = u8::is_ascii_digit;
    println!("{}", f(&b'9'));
    let g = i32::unsigned_abs;
    println!("{}", g(read(-12i32)));
    println!("{}", read(2.5f64).floor());
    println!("{:?}", [1.5f64, -2.5].iter().copied().map(f64::abs).collect::<Vec<f64>>());
    println!("{}", char::to_lowercase(read('K')).count());
    println!("{:?}", "ab".chars().map(char::to_uppercase).count());
}
