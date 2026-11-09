// As std_lang_methods_user_names, through the resolver front end (a
// module makes the root file route there).
#![allow(non_snake_case, dead_code, non_camel_case_types, non_upper_case_globals)]
mod m { pub fn k() -> u8 { 1 } }
struct E<'a>(&'a str);
struct T;
struct U(u8);
struct F;
const x: i32 = 5;
const v: u8 = 1;
static t: i32 = 2;
fn Ok() -> u8 { 3 }
trait Default { fn zz() -> Self; }
trait Clone2 {}
fn main() {
    let o: Option<u32> = Some(4);
    println!("{:?} {:?} {:?}", o.map(|q| q + 1), o.ok_or(7u8), o.and_then(|q| if q > 1 { Some(q) } else { None }));
    let r: Result<u32, String> = Err::<u32, u8>(9).map_err(|e| format!("{e}"));
    println!("{:?} {}", r.clone().ok(), r.unwrap_or_default());
    let mut m: Option<String> = None;
    m.get_or_insert_with(|| String::from("z")).push('!');
    println!("{:?} {:?}", m, m.as_deref_mut2());
    let _ = (m::k(), E("e").0, U(1).0, x, v, t, Ok());
}
trait AsDerefMut2 { fn as_deref_mut2(&self) -> usize; }
impl AsDerefMut2 for Option<String> { fn as_deref_mut2(&self) -> usize { 1 } }
