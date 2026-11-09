// K3: rustc oracle for u128.h. Prints vectors (hex lo hi pairs) for u128_test.c.
// Usage: u128_oracle [count] [seed]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200_000);
    let mut state: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0x1234_5678_9abc_def1);
    let mut next = move || {
        // splitmix64
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    let edge: [u128; 12] = [
        0, 1, 2, u64::MAX as u128, 1 << 64, (1 << 64) - 1, 1 << 127, u128::MAX,
        u128::MAX - 1, 10, i128::MAX as u128, (i128::MIN as u128) + 1,
    ];
    for &a in &edge {
        for &b in &edge {
            emit_fixed(a, b, 0);
            emit_fixed(a, b, 63);
            emit_fixed(a, b, 127);
        }
    }
    for _ in 0..count {
        let a = ((next() as u128) << 64) | next() as u128;
        let b_raw = ((next() as u128) << 64) | next() as u128;
        // Mix small divisors in: they hit the common div paths hard in practice.
        let b = if next() % 3 == 0 { b_raw % 1_000_003 } else { b_raw };
        let sh = (next() % 128) as u32;
        emit_fixed(a, b, sh);
    }
}

fn pair(v: u128) -> String {
    format!("{:x} {:x}", v as u64, (v >> 64) as u64)
}

fn emit_fixed(a: u128, b: u128, sh: u32) {
    let (ia, ib) = (a as i128, b as i128);
    let (udiv, urem) = if b == 0 { (0, 0) } else { (a / b, a % b) };
    let (idiv, irem) = if ib == 0 {
        (0i128, 0i128)
    } else {
        (ia.wrapping_div(ib), ia.wrapping_rem(ib))
    };
    let ucmp = match a.cmp(&b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    let icmp = match ia.cmp(&ib) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    println!(
        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        pair(a), pair(b), sh,
        pair(a.wrapping_add(b)), pair(a.wrapping_sub(b)), pair(a.wrapping_mul(b)),
        pair(udiv), pair(urem), pair(idiv as u128), pair(irem as u128),
        pair(a.wrapping_shl(sh)), pair(a.wrapping_shr(sh)), pair(ia.wrapping_shr(sh) as u128),
        ucmp, icmp, a, ia,
    );
}
