// A call through a fn pointer: every region of its type is late-bound,
// so the callee cannot keep an argument, and the result borrows only the
// one input it may return.
fn push(_v: &mut Vec<&u8>, _x: &u8) {}

fn first(v: &[u8]) -> &u8 {
    &v[0]
}

fn sum(a: &u8, b: &u8) -> u8 {
    *a + *b
}

fn main() {
    let f: fn(&mut Vec<&u8>, &u8) = push;
    let mut v: Vec<&u8> = Vec::new();
    {
        let l = 1u8;
        f(&mut v, &l);
    }
    println!("{}", v.len());
    let g: fn(&[u8]) -> &u8 = first;
    let w = vec![3u8];
    println!("{}", g(&w));
    let h: fn(&u8, &u8) -> u8 = sum;
    let r;
    {
        let l = 4u8;
        r = h(&l, &l);
    }
    println!("{}", r);
}
