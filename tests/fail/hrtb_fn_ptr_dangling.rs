// expect: L0306
// rustc E0597: `for<'a> fn(&'a u8) -> &'a u8` returns its argument's
// lifetime, so the result cannot outlive the local it borrows.
fn ap(f: for<'a> fn(&'a u8) -> &'a u8) -> u8 { let r; { let x = 5; r = f(&x); } *r }
fn id(x: &u8) -> &u8 { x }
fn main() { println!("{}", ap(id)); }
