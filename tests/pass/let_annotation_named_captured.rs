// ts-rust ls/crossproject.rs run_inline_item: a `let` annotation that names the impl's lifetime
// (`Option<ItemLs<'a>>`) on a variable a closure captures and writes.
enum ItemLs<'a> { Caller(&'a String), Owned(String) }
impl ItemLs<'_> { fn get(&self) -> &String { match self { ItemLs::Caller(s) => s, ItemLs::Owned(s) => s } } }
struct Q<'a> { ls: Option<&'a String> }
impl<'a> Q<'a> {
    fn run(&self) -> Option<ItemLs<'a>> {
        let mut item_ls: Option<ItemLs<'a>> = None;
        let n = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> usize {
            let ls = match self.ls {
                Some(ls) => item_ls.insert(ItemLs::Caller(ls)),
                None => item_ls.insert(ItemLs::Owned("own".to_string())),
            }
            .get();
            ls.len()
        }));
        println!("{:?}", n.ok());
        item_ls
    }
}
fn main() { let s = "abc".to_string(); let q = Q { ls: Some(&s) }; println!("{}", q.run().unwrap().get()); let q2 = Q { ls: None }; println!("{}", q2.run().unwrap().get()); }
