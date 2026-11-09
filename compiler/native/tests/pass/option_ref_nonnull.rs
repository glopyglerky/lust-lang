// `Option<&T>` values whose `Some` holds a reference release C states is nonnull (slice and
// Vec get, first, last and their _mut forms, `Some(r)` of a local reference), next to the
// None cases: out of range, empty, zero-sized elements, statics, and a matched `get` with a
// slow path, as a hot column read does.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pt {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Z;

fn pick(v: &[Pt], i: usize) -> Option<&Pt> {
    v.get(i)
}

fn kind_of(kinds: &[u16], i: usize, slow: impl Fn(usize) -> u16) -> u16 {
    match kinds.get(i) {
        Some(&k) => k,
        None => slow(i),
    }
}

fn first_last(v: &[String]) -> (Option<&String>, Option<&String>) {
    (v.first(), v.last())
}

fn some_ref<'a>(flag: bool, a: &'a Pt, b: &'a Pt) -> Option<&'a Pt> {
    let r = if flag { a } else { b };
    Some(r)
}

fn maybe(flag: bool, a: &Pt) -> Option<&Pt> {
    if flag { Some(a) } else { None }
}

static S: Pt = Pt { x: 9, y: 9 };

fn main() {
    let pts = vec![Pt { x: 1, y: 2 }, Pt { x: 3, y: 4 }];
    for i in 0..4 {
        println!("pick {i}: {:?}", pick(&pts, i));
    }
    let kinds: Vec<u16> = (0..10).map(|i| i * 3).collect();
    let mut total = 0u64;
    for i in 0..20 {
        total += kind_of(&kinds, i, |j| j as u16 + 1000) as u64;
    }
    println!("kinds {total}");
    let empty: Vec<u16> = Vec::new();
    println!("empty {:?} {:?} {:?}", empty.get(0), empty.first(), empty.last());
    let strs = vec!["a".to_string(), "bc".to_string()];
    println!("{:?}", first_last(&strs));
    println!("{:?}", first_last(&[]));
    let zs = vec![Z, Z, Z];
    println!("zst {:?} {:?} {:?} {}", zs.get(1), zs.get(3), zs.last(), zs.get(2).is_some());
    let zarr: [Z; 0] = [];
    println!("zst empty {:?}", zarr.first());
    let mut m = vec![10, 20, 30];
    if let Some(x) = m.get_mut(1) {
        *x += 1;
    }
    if let Some(x) = m.first_mut() {
        *x += 2;
    }
    if let Some(x) = m.last_mut() {
        *x += 3;
    }
    let past = m.get_mut(5).is_none();
    println!("mut {:?} {past}", m);
    let (a, b) = (Pt { x: 5, y: 6 }, Pt { x: 7, y: 8 });
    println!("some {:?} {:?} {:?} {:?}", some_ref(true, &a, &b), some_ref(false, &a, &b), maybe(true, &a), maybe(false, &a));
    let o: Option<&Pt> = Some(&S);
    println!("static {:?} {}", o.map(|p| p.x + p.y), o.is_some());
    let unit = ();
    let ou: Option<&()> = Some(&unit);
    println!("unit {:?} {}", ou, ou.is_some());
    let opts: Vec<Option<&Pt>> = (0..4).map(|i| pts.get(i)).collect();
    println!("collected {}", opts.iter().filter(|o| o.is_some()).count());
    let slice = &pts[..];
    let (mut n, mut i) = (0, 0);
    while let Some(p) = slice.get(i) {
        n += p.x;
        i += 1;
    }
    println!("while-let {n}");
}
