// A promoted reference keeps static storage in a fn whose body also has an
// iterator builtin (next, sum, collect, a for loop) or a thread_local call:
// cgen expands those calls in a copy of the body, and the copy lost the list
// of promoted temps, so `&7` lived in the dead frame. The caller clobbers
// the stack before it prints.
#[derive(Debug)]
struct P {
    a: u8,
    b: u16,
}
#[derive(Debug)]
struct W<'a, X: ?Sized> {
    n: u32,
    r: &'a X,
}
const C: u16 = 9;
thread_local! { static K: u32 = 3; }

fn pair(it: Vec<u32>) -> (Option<u32>, &'static u8) {
    (it.into_iter().next(), &7)
}
fn let_first(it: Vec<u32>) -> (Option<u32>, &'static u16) {
    let a = it.into_iter().next();
    (a, &C)
}
fn summed(v: Vec<u32>) -> W<'static, [u8]> {
    W { n: v.iter().sum(), r: &[4, 5] }
}
fn elided(v: Vec<u32>, _s: &u8) -> (usize, &P) {
    (v.iter().copied().collect::<Vec<u32>>().len(), &P { a: 3, b: 4 })
}
fn looped(v: Vec<u32>) -> Result<(u32, &'static Option<String>), String> {
    let mut s = 0;
    for x in v.iter() {
        s += x;
    }
    if s == 0 {
        return Err(String::from("empty"));
    }
    Ok((s, &None))
}
fn tls() -> [&'static (u8, char); 2] {
    let n = K.with(|k| *k);
    if n > 100 { [&(0, 'a'), &(0, 'b')] } else { [&(1, 'z'), &(2, 'y')] }
}
fn in_closure(v: Vec<u32>) -> (u32, &'static [u16; 3]) {
    let c = move || (v.iter().map(|x| x * 2).count() as u32, &[1, 2, 3]);
    c()
}

#[inline(never)]
fn clobber() -> u64 {
    let mut a = [0x5au8; 512];
    for i in 0..512 {
        a[i] = (i * 7) as u8 ^ 0xa5;
    }
    a.iter().map(|&x| x as u64).sum()
}

fn main() {
    let s = 0u8;
    let a = pair(vec![1]);
    let b = let_first(vec![2]);
    let c = summed(vec![1, 2, 3]);
    let d = elided(vec![1, 2], &s);
    let e = looped(vec![5]);
    let f = tls();
    let g = in_closure(vec![1, 2]);
    let k = clobber();
    println!("{a:?} {b:?} {c:?} {d:?}");
    println!("{e:?} {f:?} {g:?} {k}");
}
