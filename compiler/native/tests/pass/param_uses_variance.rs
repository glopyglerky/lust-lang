// A parameter counts as used through any builtin type, a fn pointer,
// a dyn bound, a projection, or another user type that uses it.
trait Tr {
    type A;
}
impl Tr for u8 {
    type A = u16;
}
struct Wrap<U>(U);
struct ViaWrap<T>(Wrap<T>);
struct ViaFn<T>(fn(T) -> i32);
struct ViaRet<T>(fn() -> T);
trait Call<X> {
    fn call(&self, x: X) -> i32;
}
struct AddOne;
impl Call<u8> for AddOne {
    fn call(&self, x: u8) -> i32 {
        x as i32 + 1
    }
}
struct ViaDyn<'a, T>(&'a dyn Call<T>);
struct ViaProj<T: Tr>(<T as Tr>::A);
struct ViaRef<'a, T>(&'a T);
struct Rec<T>(Option<Box<Rec<T>>>, T);
enum OneVariant<T> {
    Empty,
    Full(Vec<T>),
}
struct Mutual<T>(Option<Box<Other<T>>>);
struct Other<T>(Mutual<T>, Option<T>);

fn twice(x: u8) -> i32 {
    x as i32 * 2
}
fn make() -> u8 {
    7
}

fn main() {
    let a = ViaWrap(Wrap(5u8));
    let b: ViaFn<u8> = ViaFn(twice);
    let c: ViaRet<u8> = ViaRet(make);
    let add = AddOne;
    let d: ViaDyn<u8> = ViaDyn(&add);
    let e = 300;
    let _unbuilt: Option<ViaProj<u8>> = None;
    let v = 9u32;
    let f = ViaRef(&v);
    let g = Rec(Some(Box::new(Rec(None, 1))), 2);
    let h: OneVariant<i32> = OneVariant::Full(vec![1, 2]);
    let i: Mutual<u8> = Mutual(None);
    let n = match h {
        OneVariant::Empty => 0,
        OneVariant::Full(v) => v.len(),
    };
    println!("{} {} {} {} {} {} {} {} {}", (a.0).0, (b.0)(4), (c.0)(), d.0.call(1), e, f.0, g.1, n, i.0.is_none());
}
