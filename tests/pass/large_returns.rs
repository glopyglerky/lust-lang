// Functions that return aggregates over 32 bytes take the hidden
// out-pointer in the C ABI (ARCH 4.5). Output must match rustc for
// nested calls, returns of returned values, enums with large
// payloads, and drops of owned fields inside large returns.

struct Wide {
    a: i64,
    b: i64,
    c: i64,
    d: i64,
    e: i64,
    tag: String,
}

fn make(seed: i64) -> Wide {
    Wide { a: seed, b: seed + 1, c: seed + 2, d: seed + 3, e: seed + 4, tag: String::from("wide") }
}

fn sum_wide(w: Wide) -> i64 {
    w.a + w.b + w.c + w.d + w.e
}

fn relay(seed: i64) -> Wide {
    make(seed * 2)
}

enum Big {
    Small(i64),
    Large(Wide),
}

fn pick(big: bool, seed: i64) -> Big {
    if big {
        Big::Large(make(seed))
    } else {
        Big::Small(seed)
    }
}

fn total(b: Big) -> i64 {
    match b {
        Big::Small(x) => x,
        Big::Large(w) => sum_wide(w),
    }
}

struct Pair {
    first: Wide,
    second: Wide,
}

fn pair(seed: i64) -> Pair {
    Pair { first: make(seed), second: relay(seed) }
}

fn main() {
    let w = make(10);
    println!("{} {}", sum_wide(make(1)), w.tag);
    println!("{}", sum_wide(relay(5)));
    println!("{} {}", total(pick(true, 3)), total(pick(false, 7)));
    let p = pair(2);
    println!("{} {}", sum_wide(p.first), sum_wide(p.second));
    let mut acc = 0;
    let mut i = 0;
    while i < 3 {
        acc = acc + total(pick(i != 1, i));
        i = i + 1;
    }
    println!("{}", acc);
}
