// rustc promotes `&CONST` and `&expr` by the value's qualifs, not by its
// type: a value that holds no drop and no interior mutability is promoted
// even when its type needs drop (`None::<String>`). For a const item the
// qualifs come from its initializer: if and match join every arm, a call
// qualifies by its type, a trait associated const is not looked into.
use std::cell::Cell;
use std::ptr::addr_eq;

struct D(u8);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}
#[allow(dead_code)]
enum E {
    A,
    B(String),
}
#[allow(dead_code)]
struct S {
    a: u8,
    b: Option<String>,
}
struct W;
impl W {
    const N: Option<String> = None;
}

const A: Option<String> = None;
const B: Option<String> = A;
const LET: Option<String> = {
    let x = None;
    x
};
const M: Option<String> = match 1 {
    1 => None,
    _ => None,
};
const T: (Option<String>, u8) = (None, 1);
const AR: [Option<String>; 2] = [None, None];
const RS: Result<u8, String> = Ok(1);
const EA: E = E::A;
const SL: S = S { a: 1, b: None };
const OD: Option<D> = None;
const SD: Option<D> = Some(D(1));
const OC: Option<Cell<u8>> = None;
const OV: Option<Vec<u8>> = None;
const DEAD: Option<String> = if true { None } else { Some(String::new()) };
const fn mk() -> Option<String> {
    None
}
const CALL: Option<String> = mk();

fn main() {
    let r: &'static Option<String> = &A;
    let r2: &'static Option<String> = &None;
    let r3: &'static (Option<String>, u8) = &(None, 2);
    let r4: &'static Option<Cell<u8>> = &OC;
    let r5: &'static &Option<String> = &&B;
    let r6: &'static Option<String> = &W::N;
    let r7: &'static u8 = &T.1;
    println!("{} {} {} {} {} {} {}", r.is_none(), r2.is_none(), r3.1, r4.is_none(), r5.is_none(), r6.is_none(), r7);
    println!("{} {} {} {}", addr_eq(&A, &A), addr_eq(&A, &B), addr_eq(&LET, &M), addr_eq(&T, &T));
    println!("{} {} {} {}", addr_eq(&AR, &AR), addr_eq(&RS, &RS), addr_eq(&EA, &EA), addr_eq(&SL, &SL));
    println!("{} {} {}", addr_eq(&OD, &OD), addr_eq(&OC, &OC), addr_eq(&OV, &OV));
    // Not promoted: a dead arm that drops, a call, a value that drops.
    println!("{} {} {}", addr_eq(&DEAD, &DEAD), addr_eq(&CALL, &CALL), addr_eq(&SD, &SD));
    {
        let _d = &SD;
        println!("in");
    }
    println!("out");
}
