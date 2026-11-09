// rustc oracle for the 128-bit runtime helpers. Prints one line per
// operation over an edge grid plus a deterministic random stream
// (splitmix64, fixed seed). harness.c prints the same lines from the
// C helpers; test.sh compares byte for byte. The `checked` mode takes
// an op name and two decimal i128-range operands as lo/hi u64 pairs
// and prints the result or OVERFLOW, for panic-path checks.

fn edges() -> Vec<u128> {
    let mut v = vec![
        0, 1, 2, 3, 10, 0xFFFF_FFFF, 0x1_0000_0000, 0xFFFF_FFFF_FFFF_FFFF,
        0x1_0000_0000_0000_0000, u128::MAX, u128::MAX - 1,
        1 << 127, (1 << 127) - 1, 10u128.pow(19), 10u128.pow(38),
        0x8000_0000_0000_0000_0000_0000_0000_0000,
        0xDEAD_BEEF_1234_5678_9ABC_DEF0_0BAD_F00D,
    ];
    v.push(340282366920938463463374607431768211455); // u128::MAX again via digits
    v
}

fn show(a: u128, b: u128) {
    let (ia, ib) = (a as i128, b as i128);
    println!("a={a} b={b}");
    println!("uadd={} usub={} umul={}", a.wrapping_add(b), a.wrapping_sub(b), a.wrapping_mul(b));
    println!("and={} or={} xor={} not={}", a & b, a | b, a ^ b, !a);
    if b != 0 {
        println!("udiv={} urem={}", a / b, a % b);
    }
    if ib != 0 && !(ia == i128::MIN && ib == -1) {
        println!("idiv={} irem={}", ia / ib, ia % ib);
    }
    let k = (b % 128) as u32;
    println!("shl={} ushr={} ishr={}", a.wrapping_shl(k), a.wrapping_shr(k), ia.wrapping_shr(k));
    println!("ucmp={} icmp={}", cmp_u(a, b), cmp_i(ia, ib));
    println!("fmtu={a} fmti={ia} neg={}", ia.wrapping_neg());
}

fn cmp_u(a: u128, b: u128) -> i32 {
    match a.cmp(&b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn cmp_i(a: i128, b: i128) -> i32 {
    match a.cmp(&b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "checked" {
        // checked <op> <a_lo> <a_hi> <b_lo> <b_hi>   (u64 decimal each)
        let p = |i: usize| args[i].parse::<u64>().unwrap() as u128;
        let a = p(3) | (p(4) << 64);
        let b = p(5) | (p(6) << 64);
        let (ia, ib) = (a as i128, b as i128);
        let out = match args[2].as_str() {
            "uadd" => a.checked_add(b).map(|v| v.to_string()),
            "usub" => a.checked_sub(b).map(|v| v.to_string()),
            "umul" => a.checked_mul(b).map(|v| v.to_string()),
            "udiv" => a.checked_div(b).map(|v| v.to_string()),
            "urem" => a.checked_rem(b).map(|v| v.to_string()),
            "iadd" => ia.checked_add(ib).map(|v| v.to_string()),
            "isub" => ia.checked_sub(ib).map(|v| v.to_string()),
            "imul" => ia.checked_mul(ib).map(|v| v.to_string()),
            "idiv" => ia.checked_div(ib).map(|v| v.to_string()),
            "irem" => ia.checked_rem(ib).map(|v| v.to_string()),
            "ineg" => ia.checked_neg().map(|v| v.to_string()),
            "iabs" => ia.checked_abs().map(|v| v.to_string()),
            "upow" => a.checked_pow(p(5) as u32).map(|v| v.to_string()),
            "ipow" => ia.checked_pow(p(5) as u32).map(|v| v.to_string()),
            _ => None,
        };
        println!("{}", out.as_deref().unwrap_or("OVERFLOW"));
        return;
    }
    // Layout facts the C types must match.
    println!("size={} align={}", std::mem::size_of::<u128>(), std::mem::align_of::<u128>());
    let e = edges();
    for &a in &e {
        for &b in &e {
            show(a, b);
        }
    }
    let mut s: u64 = 0x1234_5678_9ABC_DEF0;
    let mut next = move || {
        s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    for _ in 0..4000 {
        let a = ((next() as u128) << 64) | next() as u128;
        let b = ((next() as u128) << 64) | next() as u128;
        show(a, b);
        // Narrow divisors hit the fast paths hard.
        show(a, b & 0xFFFF);
        show(a, b & 0xFFFF_FFFF_FFFF_FFFF);
    }
}
