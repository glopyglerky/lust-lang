// feature: attrs.cfg
// form: cfg_attr and repr(C) on a struct, as in ts-rust node layout types
#[cfg_attr(unix, derive(Debug))]
#[repr(C)]
struct Node { kind: u16, flags: u32 }
fn main() { let n = Node { kind: 1, flags: 2 }; println!("{} {} {:?}", n.kind, n.flags, std::mem::size_of::<Node>()); }
