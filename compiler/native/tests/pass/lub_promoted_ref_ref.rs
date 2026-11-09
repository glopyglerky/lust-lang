// `&&5`, `&&C` and `&&*"q"` are promoted constants, so a join that
// coerces them after their block reads no dropped temporary.
const C: u8 = 4;
fn main() {
    let v = vec![1u8, 2];
    let s = String::from("ab");
    let k = v.len() as u8;
    let a: Option<&u8> = Some(k).map(|k| match k { 0 => { &&5u8 } _ => &C });
    let b: Option<&str> = Some(k).map(|k| if k > 0 { &&*"q" } else { &s[1..] });
    let c: Option<&u8> = Some(k).map(|k| if k > 3 { &&C } else { &v[0] });
    println!("{:?} {:?} {:?}", a, b, c);
}
