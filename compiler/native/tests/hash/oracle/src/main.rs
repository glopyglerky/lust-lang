//! Rust oracle for the lust hash core. Reads an op stream from stdin
//! (see ../gen_ops.py) and prints the result plus full table state
//! (len, capacity, entries in iteration order) after every op.
//! The C harness must print the same bytes for the same stream.
//!
//! Modes: fx64, fx32, fxstr, sip64, set64 (map ops), im64, imstr
//! (index ops), fxvals (FxHasher values, no stdin), bench-fx, bench-im.
//! debug-assertions stay on so hashbrown/indexmap internal checks run.

use std::collections::HashMap;
use std::hash::{BuildHasher, BuildHasherDefault, Hash};
use std::io::{BufRead, Write};
use std::time::Instant;

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

type SipMap = HashMap<u64, u64, BuildHasherDefault<std::hash::SipHasher>>;
type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

fn make_key(kid: u64) -> String {
    let mut s = format!("k{}", kid);
    let fill = (b'a' + (kid % 26) as u8) as char;
    let reps = (kid.wrapping_mul(7) % 41) as usize;
    for _ in 0..reps {
        s.push(fill);
    }
    s
}

fn key_id(s: &str) -> u64 {
    let digits: String = s[1..].chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().unwrap()
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "fx64".into());
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    match mode.as_str() {
        "fx64" => run_map::<u64, _>(&mut out, |k| k),
        "fx32" => run_map::<u32, _>(&mut out, |k| k as u32),
        "fxstr" => run_map_str(&mut out, false),
        "sip64" => run_map_sip(&mut out),
        "set64" => run_set(&mut out),
        "im64" => run_index::<u64, _>(&mut out, |k| k, |k| *k),
        "imstr" => run_index::<String, _>(&mut out, make_key, |k| key_id(k)),
        "hb8" => run_hb_table(&mut out),
        "fxvals" => fxvals(&mut out),
        "bench-fx" => bench_fx(&mut out),
        "bench-im" => bench_im(&mut out),
        other => panic!("unknown mode {other}"),
    }
}

fn each_op(mut f: impl FnMut(&str, &mut std::str::SplitWhitespace<'_>) -> String, out: &mut impl Write) {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let Some(op) = it.next() else { continue };
        let text = f(op, &mut it);
        writeln!(out, "{text}").unwrap();
    }
}

/// Map op stream for FxHashMap<K, u64> where keys come from integers.
fn run_map<K, MK>(out: &mut impl Write, mk: MK)
where
    K: Eq + Hash + Copy + Into<u64>,
    MK: Fn(u64) -> K,
{
    run_map_generic(out, FxHashMap::default(), mk, |k: &K| (*k).into());
}

fn run_map_generic<K, MK, KF, S>(
    out: &mut impl Write,
    mut m: HashMap<K, u64, S>,
    mk: MK,
    kf: KF,
) where
    K: Eq + Hash + Copy,
    MK: Fn(u64) -> K,
    KF: Fn(&K) -> u64,
    S: BuildHasher + Default + Clone,
{
    let st = |m: &HashMap<K, u64, S>| -> String {
        let mut s = format!("{} {} [", m.len(), m.capacity());
        for (i, (k, v)) in m.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&format!("{}:{}", kf(k), v));
        }
        s.push(']');
        s
    };
    each_op(
        |op, it| {
            let num = |it: &mut std::str::SplitWhitespace<'_>| it.next().unwrap().parse::<u64>().unwrap();
            let res: String = match op {
                "n" => {
                    m = HashMap::default();
                    "new".into()
                }
                "w" => {
                    let n = num(it);
                    m = HashMap::with_capacity_and_hasher(n as usize, S::default());
                    "new".into()
                }
                "i" => {
                    let (k, v) = (mk(num(it)), num(it));
                    fmt_opt(m.insert(k, v))
                }
                "e" => {
                    let (k, v) = (mk(num(it)), num(it));
                    format!("{}", m.entry(k).or_insert(v))
                }
                "g" => fmt_opt(m.get(&mk(num(it))).copied()),
                "r" => fmt_opt(m.remove(&mk(num(it)))),
                "R" => {
                    m.reserve(num(it) as usize);
                    "ok".into()
                }
                "s" => {
                    m.shrink_to(num(it) as usize);
                    "ok".into()
                }
                "F" => {
                    m.shrink_to_fit();
                    "ok".into()
                }
                "c" => {
                    m.clear();
                    "ok".into()
                }
                "T" => {
                    let d = num(it);
                    m.retain(|_, v| *v % d != 0);
                    "ok".into()
                }
                "x" => {
                    let d = num(it);
                    let mut taken = String::new();
                    let mut first = true;
                    for (k, v) in m.extract_if(|_, v| *v % d == 0) {
                        if !first {
                            taken.push(' ');
                        }
                        first = false;
                        taken.push_str(&format!("{}:{}", kf(&k), v));
                    }
                    if first {
                        "-".into()
                    } else {
                        taken
                    }
                }
                "d" => {
                    let n = num(it);
                    let mut taken = String::new();
                    let mut first = true;
                    {
                        let mut d = m.drain();
                        for _ in 0..n {
                            match d.next() {
                                Some((k, v)) => {
                                    if !first {
                                        taken.push(' ');
                                    }
                                    first = false;
                                    taken.push_str(&format!("{}:{}", kf(&k), v));
                                }
                                None => break,
                            }
                        }
                    }
                    if first {
                        "-".into()
                    } else {
                        taken
                    }
                }
                "l" => {
                    let c = m.clone();
                    let mut s = String::new();
                    for (i, (k, v)) in c.iter().enumerate() {
                        if i > 0 {
                            s.push(' ');
                        }
                        s.push_str(&format!("{}:{}", kf(k), v));
                    }
                    format!("clone {} {} [{}]", c.len(), c.capacity(), s)
                }
                _ => panic!("bad map op {op}"),
            };
            format!("{res} ; {}", st(&m))
        },
        out,
    );
}

fn run_map_sip(out: &mut impl Write) {
    #[allow(deprecated)]
    let m: SipMap = HashMap::default();
    run_map_generic(out, m, |k| k, |k: &u64| *k);
}

/// FxHashMap<String, u64> with keys made from integer ids. State prints
/// the id, so both sides compare the same numbers.
fn run_map_str(out: &mut impl Write, _unused: bool) {
    let mut m: FxHashMap<String, u64> = FxHashMap::default();
    let st = |m: &FxHashMap<String, u64>| -> String {
        let mut s = format!("{} {} [", m.len(), m.capacity());
        for (i, (k, v)) in m.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&format!("{}:{}", key_id(k), v));
        }
        s.push(']');
        s
    };
    each_op(
        |op, it| {
            let num = |it: &mut std::str::SplitWhitespace<'_>| it.next().unwrap().parse::<u64>().unwrap();
            let res: String = match op {
                "n" => {
                    m = FxHashMap::default();
                    "new".into()
                }
                "w" => {
                    m = FxHashMap::with_capacity_and_hasher(num(it) as usize, Default::default());
                    "new".into()
                }
                "i" => {
                    let (k, v) = (make_key(num(it)), num(it));
                    fmt_opt(m.insert(k, v))
                }
                "e" => {
                    let (k, v) = (make_key(num(it)), num(it));
                    format!("{}", m.entry(k).or_insert(v))
                }
                "g" => fmt_opt(m.get(&make_key(num(it))).copied()),
                "r" => fmt_opt(m.remove(&make_key(num(it)))),
                "R" => {
                    m.reserve(num(it) as usize);
                    "ok".into()
                }
                "s" => {
                    m.shrink_to(num(it) as usize);
                    "ok".into()
                }
                "F" => {
                    m.shrink_to_fit();
                    "ok".into()
                }
                "c" => {
                    m.clear();
                    "ok".into()
                }
                "T" => {
                    let d = num(it);
                    m.retain(|_, v| *v % d != 0);
                    "ok".into()
                }
                "x" => {
                    let d = num(it);
                    let parts: Vec<String> =
                        m.extract_if(|_, v| *v % d == 0).map(|(k, v)| format!("{}:{}", key_id(&k), v)).collect();
                    if parts.is_empty() { "-".into() } else { parts.join(" ") }
                }
                "d" => {
                    let n = num(it) as usize;
                    let parts: Vec<String> = m.drain().take(n).map(|(k, v)| format!("{}:{}", key_id(&k), v)).collect();
                    if parts.is_empty() { "-".into() } else { parts.join(" ") }
                }
                "l" => {
                    let c = m.clone();
                    format!("clone {}", st(&c))
                }
                _ => panic!("bad map op {op}"),
            };
            format!("{res} ; {}", st(&m))
        },
        out,
    );
}

fn run_set(out: &mut impl Write) {
    let mut m: FxHashSet<u64> = FxHashSet::default();
    let st = |m: &FxHashSet<u64>| -> String {
        let mut s = format!("{} {} [", m.len(), m.capacity());
        for (i, k) in m.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&format!("{k}"));
        }
        s.push(']');
        s
    };
    each_op(
        |op, it| {
            let num = |it: &mut std::str::SplitWhitespace<'_>| it.next().unwrap().parse::<u64>().unwrap();
            // Map streams carry values too; set mode consumes and drops them.
            let res: String = match op {
                "n" => {
                    m = FxHashSet::default();
                    "new".into()
                }
                "w" => {
                    m = FxHashSet::with_capacity_and_hasher(num(it) as usize, Default::default());
                    "new".into()
                }
                "i" | "e" => {
                    let k = num(it);
                    let _v = num(it);
                    if m.insert(k) { "new".into() } else { "old".into() }
                }
                "g" => {
                    if m.contains(&num(it)) { "1".into() } else { "-".into() }
                }
                "r" => {
                    if m.remove(&num(it)) { "1".into() } else { "-".into() }
                }
                "R" => {
                    m.reserve(num(it) as usize);
                    "ok".into()
                }
                "s" => {
                    m.shrink_to(num(it) as usize);
                    "ok".into()
                }
                "F" => {
                    m.shrink_to_fit();
                    "ok".into()
                }
                "c" => {
                    m.clear();
                    "ok".into()
                }
                "T" => {
                    let d = num(it);
                    m.retain(|k| *k % d != 0);
                    "ok".into()
                }
                "x" => {
                    let d = num(it);
                    let parts: Vec<String> = m.extract_if(|k| *k % d == 0).map(|k| format!("{k}")).collect();
                    if parts.is_empty() { "-".into() } else { parts.join(" ") }
                }
                "d" => {
                    let n = num(it) as usize;
                    let parts: Vec<String> = m.drain().take(n).map(|k| format!("{k}")).collect();
                    if parts.is_empty() { "-".into() } else { parts.join(" ") }
                }
                "l" => {
                    let c = m.clone();
                    format!("clone {}", st(&c))
                }
                _ => panic!("bad set op {op}"),
            };
            format!("{res} ; {}", st(&m))
        },
        out,
    );
}

/// Index op stream for IndexMap<K, u64, FxBuildHasher>. Guarded ops
/// print "oob"/"dup" instead of panicking, and the C side does the same.
fn run_index<K, MK>(out: &mut impl Write, mk: MK, kf: fn(&K) -> u64)
where
    K: Eq + Hash + Clone,
    MK: Fn(u64) -> K,
{
    let mut m: FxIndexMap<K, u64> = FxIndexMap::default();
    let st = |m: &FxIndexMap<K, u64>| -> String {
        let mut s = format!("{} {} [", m.len(), m.capacity());
        for (i, (k, v)) in m.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&format!("{}:{}", kf(k), v));
        }
        s.push(']');
        s
    };
    let fmt_kv = |r: Option<(usize, &K, &u64)>| -> String {
        match r {
            Some((i, k, v)) => format!("{}:{}:{}", i, kf(k), v),
            None => "-".into(),
        }
    };
    each_op(
        |op, it| {
            let num = |it: &mut std::str::SplitWhitespace<'_>| it.next().unwrap().parse::<u64>().unwrap();
            let res: String = match op {
                "n" => {
                    m = FxIndexMap::default();
                    "new".into()
                }
                "w" => {
                    m = FxIndexMap::with_capacity_and_hasher(num(it) as usize, Default::default());
                    "new".into()
                }
                "i" => {
                    let (k, v) = (mk(num(it)), num(it));
                    let (i, old) = m.insert_full(k, v);
                    format!("{}:{}", i, fmt_opt(old))
                }
                "u" => {
                    let (k, v) = (mk(num(it)), num(it));
                    let i = m.len();
                    match m.entry(k) {
                        indexmap::map::Entry::Occupied(e) => format!("dup {}", e.index()),
                        indexmap::map::Entry::Vacant(e) => {
                            e.insert(v);
                            format!("{i}")
                        }
                    }
                }
                "g" => {
                    let k = mk(num(it));
                    match m.get_full(&k) {
                        Some((i, _, v)) => format!("{i}:{v}"),
                        None => "-".into(),
                    }
                }
                "G" => {
                    let i = num(it) as usize;
                    fmt_kv(m.get_index(i).map(|(k, v)| (i, k, v)))
                }
                "s" => {
                    let k = mk(num(it));
                    match m.shift_remove_full(&k) {
                        Some((i, k2, v)) => format!("{i}:{}:{v}", kf(&k2)),
                        None => "-".into(),
                    }
                }
                "x" => {
                    let k = mk(num(it));
                    match m.swap_remove_full(&k) {
                        Some((i, k2, v)) => format!("{i}:{}:{v}", kf(&k2)),
                        None => "-".into(),
                    }
                }
                "S" => {
                    let i = num(it) as usize;
                    match m.shift_remove_index(i) {
                        Some((k, v)) => format!("{}:{v}", kf(&k)),
                        None => "-".into(),
                    }
                }
                "X" => {
                    let i = num(it) as usize;
                    match m.swap_remove_index(i) {
                        Some((k, v)) => format!("{}:{v}", kf(&k)),
                        None => "-".into(),
                    }
                }
                "p" => match m.pop() {
                    Some((k, v)) => format!("{}:{v}", kf(&k)),
                    None => "-".into(),
                },
                "j" => {
                    let (idx, k, v) = (num(it) as usize, mk(num(it)), num(it));
                    if idx > m.len() {
                        "oob".into()
                    } else {
                        match m.entry(k) {
                            indexmap::map::Entry::Occupied(e) => format!("dup {}", e.index()),
                            indexmap::map::Entry::Vacant(e) => {
                                e.shift_insert(idx, v);
                                format!("{idx}")
                            }
                        }
                    }
                }
                "R" => {
                    m.reserve(num(it) as usize);
                    "ok".into()
                }
                "E" => {
                    m.reserve_exact(num(it) as usize);
                    "ok".into()
                }
                "h" => {
                    m.shrink_to(num(it) as usize);
                    "ok".into()
                }
                "F" => {
                    m.shrink_to_fit();
                    "ok".into()
                }
                "c" => {
                    m.clear();
                    "ok".into()
                }
                "T" => {
                    let d = num(it);
                    m.retain(|_, v| *v % d != 0);
                    "ok".into()
                }
                "d" => {
                    let (a, b) = (num(it) as usize, num(it) as usize);
                    if a > b || b > m.len() {
                        "oob".into()
                    } else {
                        let parts: Vec<String> = m.drain(a..b).map(|(k, v)| format!("{}:{v}", kf(&k))).collect();
                        if parts.is_empty() { "-".into() } else { parts.join(" ") }
                    }
                }
                "t" => {
                    m.truncate(num(it) as usize);
                    "ok".into()
                }
                "v" => {
                    m.reverse();
                    "ok".into()
                }
                "m" => {
                    let (a, b) = (num(it) as usize, num(it) as usize);
                    if a >= m.len() || b >= m.len() {
                        "oob".into()
                    } else {
                        m.move_index(a, b);
                        "ok".into()
                    }
                }
                "W" => {
                    let (a, b) = (num(it) as usize, num(it) as usize);
                    if a >= m.len() || b >= m.len() {
                        "oob".into()
                    } else {
                        m.swap_indices(a, b);
                        "ok".into()
                    }
                }
                _ => panic!("bad index op {op}"),
            };
            format!("{res} ; {}", st(&m))
        },
        out,
    );
}

/// Map op stream over hashbrown::HashTable directly. When this binary
/// is built with SSE2 off (`-C target-feature=-sse,-sse2`), hashbrown
/// uses its generic 8-byte group, so this mode gives a real width-8
/// oracle (the aarch64 NEON group is also 8 bytes; only the bitmask
/// packing differs, not the probe order) for a width-8 C build.
/// Hashes are FxHash of the key, supplied by the caller like in C.
fn run_hb_table(out: &mut impl Write) {
    use hashbrown::hash_table::Entry;
    let hash_key = |k: &u64| FxBuildHasher.hash_one(k);
    let hasher = |e: &(u64, u64)| FxBuildHasher.hash_one(e.0);
    let mut m: hashbrown::HashTable<(u64, u64)> = hashbrown::HashTable::new();
    let st = |m: &hashbrown::HashTable<(u64, u64)>| -> String {
        let mut s = format!("{} {} [", m.len(), m.capacity());
        for (i, (k, v)) in m.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&format!("{k}:{v}"));
        }
        s.push(']');
        s
    };
    each_op(
        |op, it| {
            let num = |it: &mut std::str::SplitWhitespace<'_>| it.next().unwrap().parse::<u64>().unwrap();
            let res: String = match op {
                "n" => {
                    m = hashbrown::HashTable::new();
                    "new".into()
                }
                "w" => {
                    m = hashbrown::HashTable::with_capacity(num(it) as usize);
                    "new".into()
                }
                "i" => {
                    let (k, v) = (num(it), num(it));
                    match m.entry(hash_key(&k), |e| e.0 == k, hasher) {
                        Entry::Occupied(mut e) => {
                            let old = e.get_mut().1;
                            e.get_mut().1 = v;
                            format!("{old}")
                        }
                        Entry::Vacant(e) => {
                            e.insert((k, v));
                            "-".into()
                        }
                    }
                }
                "e" => {
                    let (k, v) = (num(it), num(it));
                    match m.entry(hash_key(&k), |e| e.0 == k, hasher) {
                        Entry::Occupied(e) => format!("{}", e.get().1),
                        Entry::Vacant(e) => {
                            e.insert((k, v));
                            format!("{v}")
                        }
                    }
                }
                "g" => {
                    let k = num(it);
                    fmt_opt(m.find(hash_key(&k), |e| e.0 == k).map(|e| e.1))
                }
                "r" => {
                    let k = num(it);
                    match m.find_entry(hash_key(&k), |e| e.0 == k) {
                        Ok(e) => format!("{}", e.remove().0.1),
                        Err(_) => "-".into(),
                    }
                }
                "R" => {
                    m.reserve(num(it) as usize, hasher);
                    "ok".into()
                }
                "s" => {
                    m.shrink_to(num(it) as usize, hasher);
                    "ok".into()
                }
                "F" => {
                    m.shrink_to_fit(hasher);
                    "ok".into()
                }
                "c" => {
                    m.clear();
                    "ok".into()
                }
                "T" => {
                    let d = num(it);
                    m.retain(|e| e.1 % d != 0);
                    "ok".into()
                }
                "x" => {
                    let d = num(it);
                    let parts: Vec<String> =
                        m.extract_if(|e| e.1 % d == 0).map(|(k, v)| format!("{k}:{v}")).collect();
                    if parts.is_empty() { "-".into() } else { parts.join(" ") }
                }
                "d" => {
                    let n = num(it) as usize;
                    let parts: Vec<String> = m.drain().take(n).map(|(k, v)| format!("{k}:{v}")).collect();
                    if parts.is_empty() { "-".into() } else { parts.join(" ") }
                }
                "l" => {
                    let c = m.clone();
                    format!("clone {}", st(&c))
                }
                _ => panic!("bad hb op {op}"),
            };
            format!("{res} ; {}", st(&m))
        },
        out,
    );
}

/// FxBuildHasher::hash_one values for core types. The C harness prints
/// the same lines from lust_fx_* primitives, following core's Hash byte
/// streams by hand; a diff here means a byte-stream rule is wrong.
fn fxvals(out: &mut impl Write) {
    let h = FxBuildHasher;
    macro_rules! line {
        ($name:expr, $v:expr) => {
            writeln!(out, "{} {:?} {}", $name, $v, h.hash_one($v)).unwrap()
        };
    }
    for v in [0u8, 1, 100, 255] {
        line!("u8", v);
    }
    for v in [i8::MIN, -1, 0, 1, 100, i8::MAX] {
        line!("i8", v);
    }
    for v in [0u16, 1, 1000, u16::MAX] {
        line!("u16", v);
    }
    for v in [i16::MIN, -1, 0, 1, i16::MAX] {
        line!("i16", v);
    }
    for v in [0u32, 1, 100, 70000, u32::MAX] {
        line!("u32", v);
    }
    for v in [i32::MIN, -1, 0, 1, i32::MAX] {
        line!("i32", v);
    }
    for v in [0u64, 1, 100, 1 << 40, u64::MAX] {
        line!("u64", v);
    }
    for v in [i64::MIN, -1, 0, 1, i64::MAX] {
        line!("i64", v);
    }
    for v in [0u128, 1, 100, 1 << 100, u128::MAX] {
        line!("u128", v);
    }
    for v in [i128::MIN, -1, 0, 1, i128::MAX] {
        line!("i128", v);
    }
    for v in [0usize, 1, 12345, usize::MAX] {
        line!("usize", v);
    }
    for v in [isize::MIN, -1, 0, 1, isize::MAX] {
        line!("isize", v);
    }
    for v in [false, true] {
        line!("bool", v);
    }
    for v in ['a', 'Z', 'é', '\u{1F600}'] {
        writeln!(out, "char {} {}", v as u32, h.hash_one(v)).unwrap();
    }
    for s in ["", "a", "uwu", "hello world", "0123456789abcdef", "0123456789abcdefg"] {
        writeln!(out, "str {:?} {}", s, h.hash_one(s)).unwrap();
        writeln!(out, "String {:?} {}", s, h.hash_one(s.to_string())).unwrap();
    }
    let long = "x".repeat(100);
    writeln!(out, "str100 {}", h.hash_one(long.as_str())).unwrap();
    // Byte slices as &[u8] values (length prefix + one write of bytes).
    let slices: &[&[u8]] = &[&[], &[0], &[1, 2, 3], &[9u8; 40]];
    for s in slices {
        writeln!(out, "bytes {:?} {}", s, h.hash_one(s)).unwrap();
    }
    // A bare write of bytes (the rustc-hash HashBytes test shape).
    struct RawBytes<'a>(&'a [u8]);
    impl Hash for RawBytes<'_> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            state.write(self.0);
        }
    }
    for s in slices {
        writeln!(out, "rawbytes {:?} {}", s, h.hash_one(RawBytes(s))).unwrap();
    }
    for t in [(0u32, 0u64), (5, 7), (u32::MAX, 1)] {
        writeln!(out, "tuple {:?} {}", t, h.hash_one(t)).unwrap();
    }
    // Random integers and strings, same splitmix64 stream in C.
    let mut rng = 0x1234567890ABCDEFu64;
    let mut next = move || {
        rng = rng.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };
    for _ in 0..2000 {
        let v = next();
        writeln!(out, "rand-u64 {v} {}", h.hash_one(v)).unwrap();
        let kid = v % 100000;
        let s = make_key(kid);
        writeln!(out, "rand-str {kid} {}", h.hash_one(s.as_str())).unwrap();
    }
}

fn bench_fx(out: &mut impl Write) {
    let n = 200_000u64;
    let mut m: FxHashMap<u64, u64> = FxHashMap::default();
    let t = Instant::now();
    for k in 0..n {
        m.insert(k * 2654435761 % (n + n / 2), k);
    }
    let ins = t.elapsed().as_nanos() as u64;
    let t = Instant::now();
    let mut acc = 0u64;
    for k in 0..n {
        if let Some(v) = m.get(&(k * 2654435761 % (n + n / 2))) {
            acc = acc.wrapping_add(*v);
        }
        let _ = m.get(&(k + n * 2));
    }
    let get = t.elapsed().as_nanos() as u64;
    let t = Instant::now();
    for k in (0..n).step_by(2) {
        m.remove(&(k * 2654435761 % (n + n / 2)));
    }
    let rem = t.elapsed().as_nanos() as u64;
    let t = Instant::now();
    let mut total = 0u64;
    for _ in 0..20 {
        for v in m.values() {
            total = total.wrapping_add(*v);
        }
    }
    let iter = t.elapsed().as_nanos() as u64;
    writeln!(out, "rust-fx insert_ns={:.2} get2_ns={:.2} remove_ns={:.2} iter_ns={:.2} acc={} len={}",
        ins as f64 / n as f64, get as f64 / (2 * n) as f64, rem as f64 / (n / 2) as f64,
        iter as f64 / (20 * m.len().max(1)) as f64, acc ^ total, m.len())
    .unwrap();
}

fn bench_im(out: &mut impl Write) {
    let n = 200_000u64;
    let mut m: FxIndexMap<u64, u64> = FxIndexMap::default();
    let t = Instant::now();
    for k in 0..n {
        m.insert(k * 2654435761 % (n + n / 2), k);
    }
    let ins = t.elapsed().as_nanos() as u64;
    let t = Instant::now();
    let mut acc = 0u64;
    for k in 0..n {
        if let Some(v) = m.get(&(k * 2654435761 % (n + n / 2))) {
            acc = acc.wrapping_add(*v);
        }
    }
    let get = t.elapsed().as_nanos() as u64;
    writeln!(out, "rust-im insert_ns={:.2} get_ns={:.2} acc={} len={}",
        ins as f64 / n as f64, get as f64 / n as f64, acc, m.len())
    .unwrap();
}

fn fmt_opt(v: Option<u64>) -> String {
    match v {
        Some(v) => format!("{v}"),
        None => "-".into(),
    }
}
