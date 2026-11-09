// A non-capturing closure coerced to a fn pointer whose output borrows the one input may return
// borrows of its argument (ts-rust ast/fields.rs list_of! with ListSel, L0733 before).
pub enum Data {
    Doc(Option<Vec<u8>>),
    Other,
}
#[derive(Debug)]
pub enum AnyList<'a> {
    Nodes(&'a Vec<u8>),
    Mods(&'a [u8]),
}
impl<'a> AnyList<'a> {
    fn len(self) -> usize {
        match self {
            AnyList::Nodes(v) => v.len(),
            AnyList::Mods(m) => m.len(),
        }
    }
}
pub type ListSel = for<'a> fn(&'a Data) -> Option<Option<AnyList<'a>>>;

fn scoped(d: &Data, sel: ListSel) -> Option<usize> {
    sel(d).map(|o| o.map_or(0, AnyList::len))
}

fn tags(d: &Data) -> usize {
    scoped(d, |d| match d {
        Data::Doc(t) => Some(t.as_ref().map(AnyList::Nodes)),
        _ => None,
    })
    .unwrap_or(0)
}

struct Pick {
    f: fn(&[u8]) -> Option<&u8>,
}

fn main() {
    println!("{} {}", tags(&Data::Doc(Some(vec![1, 2]))), tags(&Data::Other));
    let first: fn(&Data) -> Option<AnyList<'_>> = |d| match d {
        Data::Doc(Some(v)) => Some(AnyList::Mods(&v[1..])),
        _ => None,
    };
    println!("{:?}", first(&Data::Doc(Some(vec![3, 4, 5]))));
    let p = Pick { f: |s| s.iter().max() };
    println!("{:?}", (p.f)(&[4, 9, 2]));
    let pair: fn(&(u8, String)) -> (&u8, &str) = |t| (&t.0, t.1.as_str());
    let t = (7, String::from("seven"));
    println!("{:?} {}", pair(&t), alias_use());
}

// An elided pointer type through an alias, as a struct field.
type Sel = fn(&(u8, String)) -> &str;
struct HasSel {
    f: Sel,
}
#[allow(dead_code)]
fn alias_use() -> usize {
    let h = HasSel { f: |t| t.1.as_str() };
    (h.f)(&(1, String::from("ab"))).len()
}
