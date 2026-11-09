// expect: L0110
// rustc E0308: with no element type, a later fn item must be a subtype of
// the first; `g_id` returns its argument's lifetime, not `'static`.
static S: u8 = 4;
fn g_id(x: &u8) -> Option<&u8> { Some(x) }
fn g_st(_x: &u8) -> Option<&'static u8> { Some(&S) }
fn main() { let a = [g_st, g_id]; println!("{:?}", a[1](&2)); }
