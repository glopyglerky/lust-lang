// R08: `Ref` and `RefMut` implement `Deref` (and `RefMut` `DerefMut`) in the trait system,
// so core's `Option::as_deref` and a `Deref` bound or `Target` projection see them.
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
struct Snap { options: i32 }
fn len(s: Option<&Vec<u32>>) -> usize { s.map_or(0, |v| v.len()) }
fn size<D: Deref>(d: D) -> usize where D::Target: Sized { std::mem::size_of::<D::Target>() }
fn edit<D: Deref + DerefMut>(d: &mut D) -> &mut D::Target { d.deref_mut() }
fn main() {
    let c = RefCell::new(Snap { options: 3 });
    let o = Some(&c);
    let r = o.map(|x| x.borrow());
    let ok = r.as_deref().is_some_and(|old| old.options == 3);
    drop(r);
    let mut ob = Some(c.borrow_mut());
    if let Some(s) = ob.as_deref_mut() {
        s.options += 1;
    }
    println!("{} {}", ok, ob.as_deref().map(|x| x.options).unwrap_or(0));
    drop(ob);
    let v = RefCell::new(vec![1u32, 2]);
    let g: Option<Ref<'_, Vec<u32>>> = Some(v.borrow());
    println!("{}", len(g.as_deref()));
    {
        let g = g.unwrap();
        let t: &<Ref<'_, Vec<u32>> as Deref>::Target = &*g;
        println!("{} {}", t.len(), size(v.borrow()));
    }
    let mut m: RefMut<'_, Vec<u32>> = v.borrow_mut();
    edit(&mut m).push(3);
    println!("{:?}", *m);
}
