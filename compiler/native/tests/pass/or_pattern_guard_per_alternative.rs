// A guard on an or-pattern runs once per matching combination of
// alternatives, also when the or-pattern binds nothing.
fn tick(c: &mut u32, tag: u32) -> bool { *c += 1; println!("g{}", tag); false }
fn main() {
    let mut c = 0;
    let v = Some(3);
    match v { _ | _ if tick(&mut c, 1) => {}, _ => {} }
    match v { Some(_) | Some(3) if tick(&mut c, 2) => {}, _ => {} }
    match (1, 2) { (1 | 1, 2 | _) if tick(&mut c, 3) => {}, _ => {} }
    match (1, 2) { (1 | 1, _) | (_, 2) if tick(&mut c, 4) => {}, _ => {} }
    match 5 { 1 | 5 | 5 if tick(&mut c, 5) => {}, 5 | 5 if tick(&mut c, 6) => {}, _ => {} }
    match Some(Some(1)) { Some(None | Some(_)) | Some(Some(1)) if tick(&mut c, 7) => {}, _ => {} }
    let s = "a";
    match s { "a" | "a" if tick(&mut c, 8) => {}, _ => {} }
    match 'x' { 'a'..='z' | 'x' if tick(&mut c, 9) => {}, _ => {} }
    println!("{}", c);
}
