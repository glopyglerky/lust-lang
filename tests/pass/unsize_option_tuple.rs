fn f(o: Option<&[u8]>) -> usize {
    match o {
        Some([a, b, ..]) => (*a + *b) as usize,
        Some([x]) => *x as usize,
        Some([]) => 0,
        None => 99,
    }
}

fn g(t: (&[u8], u8)) -> usize {
    t.0.len() + t.1 as usize
}

static A: [u8; 3] = [7, 8, 9];
static S: &[u8] = &A;
static T: &str = "hello";
static U: (&str, &[u8]) = ("p", &[1, 2]);

fn main() {
    let v = vec![3u8, 4, 5];
    println!("{} {} {}", f(Some(&v)), f(Some(&v[..1])), f(None));
    let a = [1u8, 2, 3];
    println!("{}", g((&a, 9)));
    println!("{:?} {} {:?}", S, T, U);
}
