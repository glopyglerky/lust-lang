// A `&dyn Fn` as `Iterator::map`'s callable (was a reject: the adapter
// expansion now calls the vtable's call entry).
fn main() { let r: &dyn Fn(i32) -> i32 = &|x| x + 1; let v: Vec<i32> = vec![1, 2].into_iter().map(r).collect(); println!("{:?}", v); }
