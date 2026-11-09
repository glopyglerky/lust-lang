// A written MapWhile type with a fn pointer predicate (R22).
fn main() { let s: std::iter::MapWhile<std::vec::IntoIter<u8>, fn(u8) -> Option<u16>> = vec![1u8].into_iter().map_while(|x| Some(x as u16)); println!("{:?}", s.collect::<Vec<_>>()); }
