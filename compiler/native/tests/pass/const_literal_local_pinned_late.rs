// An unsuffixed literal local (`let mut hash = 0;`) takes the integer type
// of its first typed use in the const evaluator: a typed call argument, a
// typed operand, an index, a typed `let`, or the return. ts-rust's
// lib_parse_snapshot.rs hashes its sources this way in a const block.
const fn mix(x: u64) -> u64 {
    x.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (x >> 31)
}

const HASH: u64 = {
    let hashes = [1u64, 2, 3, 0xFFFF_FFFF_FFFF_FFFF];
    let mut hash = 0;
    let mut i = 0;
    while i < hashes.len() {
        hash = mix(hash ^ hashes[i]);
        i += 1;
    }
    hash
};

// Written before the pin: the evaluation runs again with the type.
const IDX: usize = {
    let arr = [10usize, 20, 30, 40, 50];
    let mut i = 0;
    while i < 3 {
        i += 1;
    }
    arr[i]
};

// Two literal locals that meet share one type.
const fn sum16() -> u16 {
    let mut a = 0;
    let b = 250;
    a = a + b;
    a = a + 10;
    a
}

// A shift amount keeps its own type.
const SHIFTED: u64 = {
    let s = 3;
    let x = 1u64 << s;
    let y: u8 = s;
    x + y as u64
};

const fn pow3(n: u32) -> u64 {
    let mut acc = 1;
    let mut k = 0;
    while k < n {
        acc = acc * 3;
        k += 1;
    }
    acc
}

const TAIL: u8 = {
    let h = 5;
    h
};

const NEG: i64 = {
    let mut x = 0;
    x -= 5;
    x * 3
};

// A dead branch's write does not count.
const fn pick(x: usize) -> usize {
    let mut h = 7;
    if x > 5 {
        h = 1000;
    }
    h
}

fn main() {
    println!("{HASH} {IDX} {} {SHIFTED} {} {TAIL} {NEG}", sum16(), pow3(30));
    const P: usize = pick(0);
    println!("{P} {}", pick(9));
}
