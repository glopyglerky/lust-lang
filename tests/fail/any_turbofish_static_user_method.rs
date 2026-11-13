// expect: L0203
// rustc: E0597 `s` does not live long enough. A user method named `downcast_ref` takes its
// turbofish as written, so the `'static` there must hold.
struct S;
impl S {
    fn downcast_ref<T>(&self, t: T) -> T {
        t
    }
}
fn main() {
    let s = String::from("a");
    let r = S.downcast_ref::<&'static str>(&s);
    println!("{r}");
}
