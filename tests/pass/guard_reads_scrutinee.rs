// Guards may read the scrutinee and write other places; the scrutinee
// borrow ends with the guard, so the arm body may mutate it.
fn tick(c: &mut u32) -> bool { *c += 1; *c > 1 }
struct S { a: u8, b: Vec<u8> }
fn main() {
    let mut c = 0;
    let s: &str = "hi";
    match s { "hi" if tick(&mut c) => println!("a"), _ => println!("b") }
    let v = vec![1, 2, 3];
    let sl: &[i32] = &v;
    match *sl { [x, ..] if x > 0 => println!("{}", x), _ => {} }
    match sl { [x, rest @ ..] if rest.len() > *x as usize => println!("r"), _ => println!("nr") }
    let mut o = Some(String::from("x"));
    match o { Some(ref s) if s.len() == 1 => println!("{}", s), _ => {} }
    match &mut o { Some(s) if s.len() == 1 => s.push('y'), _ => {} }
    match &mut o { Some(s) if s.len() == 2 => s.push('z'), _ => {} }
    let mut st = S { a: 1, b: vec![] };
    match st.a { 1 if { c += 1; true } => st.b.push(1), _ => {} }
    let mut n = 0;
    for i in 0..4 {
        match i { k if k % 2 == 0 && tick(&mut c) => n += k, _ => n += 100 }
    }
    let mut it = vec![1, 2];
    while let Some(x) = it.pop() { match x { 2 if it.len() > 0 => println!("one"), _ => println!("other") } }
    println!("{:?} {} {} {:?}", o, n, c, st.b);
}
