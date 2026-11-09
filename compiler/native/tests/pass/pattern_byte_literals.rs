fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\r' | b'\n')
}

fn classify(c: u8) -> u32 {
    match c {
        b'a'..=b'z' => 1,
        b'A'..b'[' => 2,
        b'0'..=b'9' => 3,
        0..b' ' => 4,
        b'~'.. => 5,
        _ => 0,
    }
}

const fn is_ab(s: &[u8]) -> bool {
    matches!(s, b"ab")
}

const AB: bool = is_ab(b"ab");

fn main() {
    println!("{}", AB);
    // Scalar byte patterns auto-deref a reference scrutinee.
    let rc: &u8 = &b'm';
    println!("{} {}", matches!(rc, b'a'..=b'z'), matches!(rc, b'm'));
    println!("{}", match 50u8 { ..=b'z' => 1, _ => 0 });
    println!("{} {}", is_ws(b' '), is_ws(b'x'));
    println!("{} {} {} {} {}", classify(b'm'), classify(b'G'), classify(b'7'), classify(1), classify(b'~'));
    let esc: u8 = 0xff;
    println!("{} {} {} {}", matches!(esc, b'\xff'), matches!(0u8, b'\0'), matches!(92u8, b'\\'), matches!(39u8, b'\''));
    // Byte string patterns: on &[u8; N], on &[u8], nested, and empty.
    let arr: &[u8; 3] = b"abc";
    println!("{} {}", matches!(arr, b"abc"), matches!(arr, b"abd"));
    let slice: &[u8] = b"abc";
    println!("{} {}", matches!(slice, b"abc"), matches!(slice, b"ab"));
    println!("{}", matches!(slice, b"ab" | b"abc"));
    let hi: &[u8] = b"hi";
    let opt: Option<&[u8]> = Some(hi);
    println!("{}", matches!(opt, Some(b"hi")));
    let empty: &[u8] = b"";
    println!("{} {}", matches!(empty, b""), matches!(slice, b""));
    let raw: &[u8; 2] = br"ab";
    println!("{} {}", matches!(raw, br"ab"), matches!(raw, br#"ab"#));
    let nl: &[u8] = b"a\n";
    println!("{}", matches!(nl, b"a\x0a"));
    // Byte char patterns inside slice and tuple patterns.
    let word: &[u8] = b"abcd";
    println!("{} {}", matches!(word, [b'a', b'b', ..]), matches!(word, [b'a', .., b'd']));
    let pair: [u8; 2] = [97, 98];
    println!("{}", matches!(pair, [b'a', b'b']));
    let tup: (u8, u8) = (97, 98);
    println!("{}", matches!(tup, (b'a', b'b')));
    // Refutable positions rustc accepts: if let, let-else, @ binding.
    let c: u8 = b'\n';
    if let b'\n' = c {
        println!("newline");
    }
    let b'z' = b'z' else {
        println!("not z");
        return;
    };
    println!("z ok");
    if let x @ b'a' = tup.0 {
        println!("at {x}");
    }
    let (b'a', second) = tup else {
        println!("tuple else");
        return;
    };
    println!("second {second}");
}
