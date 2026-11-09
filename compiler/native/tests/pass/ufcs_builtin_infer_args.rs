// A user trait's assoc fn through a builtin type path that omits the
// type's args (`Vec::from_json(v)`, ts_ast_codegen lib.rs:175): rustc
// infers them, as for `<Vec<_>>::from_json(v)`.
trait FromNum: Sized {
    fn from_num(v: &i32) -> Result<Self, String>;
}
impl FromNum for i32 {
    fn from_num(v: &i32) -> Result<Self, String> {
        Ok(*v)
    }
}
impl<T: FromNum> FromNum for Vec<T> {
    fn from_num(v: &i32) -> Result<Self, String> {
        Ok(vec![T::from_num(v)?, T::from_num(&(v + 1))?])
    }
}
impl<T: FromNum> FromNum for Option<T> {
    fn from_num(v: &i32) -> Result<Self, String> {
        if *v == 0 { Ok(None) } else { T::from_num(v).map(Some) }
    }
}
impl<T: FromNum> FromNum for Box<T> {
    fn from_num(v: &i32) -> Result<Self, String> {
        T::from_num(v).map(Box::new)
    }
}

#[derive(Debug)]
enum Alias {
    Members(Vec<i32>),
    One(Option<i32>),
}

fn main() {
    let a = Vec::from_num(&3).map(Alias::Members);
    let b = Option::from_num(&4).map(Alias::One);
    let c: Result<Vec<i32>, String> = Vec::from_num(&5);
    let d: Result<Box<i32>, String> = Box::from_num(&6);
    let e: Option<Option<i32>> = Option::from_num(&0).ok();
    println!("{a:?} {b:?} {c:?} {d:?} {e:?}");
}
