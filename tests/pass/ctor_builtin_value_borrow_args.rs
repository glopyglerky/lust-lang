// Builtin constructors and generic fn items as values with a borrowed
// input: at a generic Fn bound, in Option/Result `map`, and in iterator
// adapters (ts_ast_codegen json.rs: `T::from_json(v).map(Some)`).
use std::rc::Rc;

trait FromNum: Sized {
    fn from_num(v: &i32) -> Result<Self, String>;
}
impl FromNum for i32 {
    fn from_num(v: &i32) -> Result<Self, String> {
        if *v < 0 { Err(format!("negative {v}")) } else { Ok(*v) }
    }
}
impl<T: FromNum> FromNum for Option<T> {
    fn from_num(v: &i32) -> Result<Self, String> {
        match v {
            0 => Ok(None),
            _ => T::from_num(v).map(Some),
        }
    }
}

fn keep<T, R, F: Fn(T) -> R>(f: F, x: T) -> R {
    f(x)
}
fn id<T>(x: T) -> T {
    x
}
fn get<'a>(d: &'a [u8]) -> Vec<Box<&'a u8>> {
    d.iter().map(Box::new).collect()
}

fn main() {
    println!("{:?} {:?} {:?}", <Option<i32>>::from_num(&3), <Option<i32>>::from_num(&0), <Option<i32>>::from_num(&-1));
    let data = vec![1u8, 2];
    let o = keep(Some, &data[0]);
    let r = keep(Ok::<_, ()>, data.as_slice());
    let b = keep(Rc::new, &data[1]);
    let i = keep(id, &data[1]);
    println!("{o:?} {r:?} {b:?} {i:?}");
    let s = String::from("ab");
    let e: Result<u8, &str> = Err(s.as_str()).map_err(Err::<u8, &str>).unwrap_or_else(id);
    println!("{e:?} {:?}", get(&data));
    let words: Vec<Option<&str>> = s.split('b').map(Some).collect();
    println!("{words:?}");
    drop(data);
}
