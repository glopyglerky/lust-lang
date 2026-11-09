// expect: L0611
// rustc E0275: the impl for Option asks `T: Tr` one level down, 128 times.
trait Tr {
    fn d(&self) -> u32;
}
impl Tr for u8 {
    fn d(&self) -> u32 {
        0
    }
}
impl<T: Tr> Tr for Option<T> {
    fn d(&self) -> u32 {
        1
    }
}
fn main() {
    let x: Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<u8>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> = Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(1u8))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))));
    println!("{}", x.d());
}
