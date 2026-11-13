// core's [T]::split_at family: both halves borrow the receiver at once, and a
// mem::take of a `&'static mut [u8]` field gives the field's own lifetime (ts-rust's interner).
// The leaked blocks stay reachable from a static, so the sanitized build sees no leak.
static KEEP: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
struct Shard { free: &'static mut [u8] }
fn intern(shard: &mut Shard, s: &str) -> &'static str {
    if shard.free.len() < s.len() {
        shard.free = Box::leak(vec![0u8; 8.max(s.len())].into_boxed_slice());
    }
    let (head, tail) = std::mem::take(&mut shard.free).split_at_mut(s.len());
    head.copy_from_slice(s.as_bytes());
    shard.free = tail;
    let head: &'static [u8] = head;
    std::str::from_utf8(head).expect("utf8")
}
fn main() {
    let mut v = vec![1u8, 2, 3, 4];
    let (a, b) = v.split_at_mut(1);
    a[0] = 9; b[0] = 8; b[2] += a[0];
    println!("{v:?}");
    let (x, y) = v.split_at(4); println!("{x:?} {y:?}");
    if let Some((f, r)) = v.split_first_mut() { *f = 1; r[0] = 2; }
    if let Some((l, r)) = v.split_last_mut() { *l = 7; r[1] = 6; }
    println!("{v:?} {:?} {:?}", v.split_first(), v.split_last());
    let e: [u8; 0] = []; println!("{:?} {:?}", e.split_first(), e.split_last());
    let mut arr = [1, 2, 3]; let (p, q) = arr.split_at_mut(3); p[0] = 0; println!("{p:?} {q:?}");
    println!("{:?} {:?}", arr.split_at_checked(4), arr.split_at_checked(0));
    let s: &mut [i32] = &mut arr[..]; if let Some((m, n)) = s.split_at_mut_checked(2) { m[1] = n[0]; } println!("{arr:?}");
    let mut sh = Shard { free: &mut [] };
    let w1 = intern(&mut sh, "hello"); let w2 = intern(&mut sh, "abc"); let w3 = intern(&mut sh, "z");
    println!("{w1} {w2} {w3} {}", sh.free.len());
    KEEP.lock().unwrap().extend([w1, w2, w3]);
    let st = vec![String::from("a"), String::from("b")];
    let (l, r) = st.split_at(1); println!("{l:?} {r:?}");
}
