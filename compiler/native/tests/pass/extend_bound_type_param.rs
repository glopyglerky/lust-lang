// extend on a type parameter through an Extend<A> bound (ts-rust frontend/core_ls_ext.rs:249 copy_map_into).
use std::collections::HashMap;
pub fn copy_map_into<M>(dst: Option<M>, src: &M) -> M where M: Clone + IntoIterator + Extend<<M as IntoIterator>::Item> {
    match dst { None => src.clone(), Some(mut dst) => { dst.extend(src.clone()); dst } }
}
fn main() { let mut a = HashMap::new(); a.insert(1u8, 2u8); let mut b = HashMap::new(); b.insert(3u8, 4u8); let c = copy_map_into(Some(b), &a); let mut v: Vec<_> = c.into_iter().collect(); v.sort(); println!("{:?}", v); }
